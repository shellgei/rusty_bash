//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::core::data::Value;
use crate::utils::error;
use crate::utils::exit;
use std::env;
use super::array::Array;
use super::subscript::Subscript;
use super::word::Word;

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    pub name: String,
    pub index: Option<Subscript>,
    pub value: Value,
    pub evaluated_value: Value,
    pub append: bool,
}

fn readonly_error(name: &str, core: &mut ShellCore) -> bool {
    core.data.set_param("?", "1");
    let msg = error::readonly(name);
    error::print(&msg, core);
    false
}

fn bad_subscript_error(sub: &str, core: &mut ShellCore) -> bool {
    core.data.set_param("?", "1");
    let msg = error::bad_array_subscript(&sub);
    error::print(&msg, core);
    false
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore,
                local: bool, env: bool) -> bool {
        self.evaluated_value = match self.value.clone() {
            Value::None      => Value::EvaluatedSingle("".to_string()),
            Value::Single(v) => self.eval_as_value(&v, core),
            Value::Array(a)  => self.eval_as_array(&mut a.clone(), core),
            _                => return false,
        };

        match env {
            false => self.set_to_shell(core, local),
            true  => self.set_to_env(),
        }
    }

    fn set_assoc(&mut self, core: &mut ShellCore, local: bool) -> bool {
        let index = self.get_index(core);
        let result = match (&self.evaluated_value, index, local) {
            (Value::EvaluatedSingle(v), Some(k), false) 
              => core.data.set_assoc_elem(&self.name, &k, v),
            (Value::EvaluatedSingle(v), Some(k), true) 
              => core.data.set_local_assoc_elem(&self.name, &k, v),
            _ => return bad_subscript_error(&self.text, core),
        };
        if ! result {
            readonly_error(&self.name, core);
            return false;
        }
        true
    }

    fn set_array(&mut self, core: &mut ShellCore, local: bool) -> bool {
        let index = match self.get_index(core) {
            Some(s) => {
                match s.parse::<usize>() {
                    Ok(n) => Some(n),
                    _ => return bad_subscript_error(&self.text, core),
                }
            },
            None => None,
        };

        let result = match (&self.evaluated_value, index, local) {
            (Value::EvaluatedSingle(v), Some(n), true) 
                => core.data.set_local_array_elem(&self.name, v, n),
            (Value::EvaluatedSingle(v), Some(n), false) 
                => core.data.set_array_elem(&self.name, v, n),
            (_, Some(_), _) 
                => false,
            (Value::EvaluatedArray(a), None, true) 
                => core.data.set_local_array(&self.name, &a),
            (Value::EvaluatedArray(a), None, false) 
                => core.data.set_array(&self.name, &a),
            _ => exit::internal("Unknown variable"),
        };

        match result {
            true  => true,
            false => readonly_error(&self.name, core),
        }
    }
 
    fn set_param(&mut self, core: &mut ShellCore, local: bool) -> bool {
        let result = match (&self.evaluated_value, local) {
            (Value::EvaluatedSingle(v), true)
                => core.data.set_local_param(&self.name, &v),
            (Value::EvaluatedSingle(v), false)
                => core.data.set_param(&self.name, &v),
            (Value::EvaluatedArray(a), true) 
                => core.data.set_local_array(&self.name, &a),
            (Value::EvaluatedArray(a), false) 
                => core.data.set_array(&self.name, &a),
            _ => exit::internal("Unknown variable"),
        };

        match result {
            true  => true,
            false => readonly_error(&self.name, core),
        }
    }

    fn set_to_shell(&mut self, core: &mut ShellCore, local: bool) -> bool {
        match &self.evaluated_value {
            Value::None => {
                core.data.set_param("?", "1");
                return false;
            },
            _ => {},
        }

        if core.data.is_assoc(&self.name) {
            self.set_assoc(core, local)
        }else if core.data.is_array(&self.name) {
            self.set_array(core, local)
        }else {
            self.set_param(core, local)
        }
    }

    pub fn set_to_env(&mut self) -> bool {
        match &self.evaluated_value {
            Value::EvaluatedSingle(v) => env::set_var(&self.name, &v),
            _ => return false,
        }
        true
    }

    pub fn get_index(&mut self, core: &mut ShellCore) -> Option<String> {
        match self.index.clone() {
            Some(mut s) => {
                if s.text.chars().all(|c| " \n\t[]".contains(c)) {
                    return Some("".to_string());
                }
                s.eval(core, &self.name)
            },
            _ => None,
        }
    }

    fn eval_as_value(&self, w: &Word, core: &mut ShellCore) -> Value {
        let prev = match self.append {
            true  => core.data.get_param(&self.name),
            false => "".to_string(),
        };

        match w.eval_as_value(core) {
            Some(s) => Value::EvaluatedSingle(prev + &s),
            None    => Value::None,
        }
    }

    fn eval_as_array(&self, a: &mut Array, core: &mut ShellCore) -> Value {
        let prev = match self.append {
            true  => core.data.get_array_all(&self.name),
            false => vec![],
        };

        match a.eval(core) {
            Some(values) => Value::EvaluatedArray([prev, values].concat()),
            None         => Value::None,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return None;
        }

        let mut ans = Self::default();

        feeder.set_backup();
        let name = feeder.consume(len);
        ans.name = name.clone();
        ans.text += &name;

        if let Some(s) = Subscript::parse(feeder, core) {
            ans.text += &s.text.clone();
            ans.index = Some(s);
        };

        if feeder.starts_with("+=") {
            ans.append = true;
            ans.text += &feeder.consume(2);
        }else if feeder.starts_with("=") {
            ans.text += &feeder.consume(1);
        }else {
            feeder.rewind();
            return None;
        }
        feeder.pop_backup();

        if let Some(a) = Array::parse(feeder, core) {
            ans.text += &a.text;
            ans.value = Value::Array(a);
            Some(ans)
        }else if let Some(w) = Word::parse(feeder, core, false) {
            ans.text += &w.text;
            ans.value = Value::Single(w);
            Some(ans)
        }else {
            Some(ans)
        }
    }
}
