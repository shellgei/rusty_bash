//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::data::DataType;
use crate::data::single::SingleData;
use crate::utils::error;
use std::env;
use super::array::Array;
use super::subscript::Subscript;
use super::word::Word;
use crate::data::array::ArrayData;

#[derive(Debug, Clone, Default)]
pub enum ParsedDataType {
    #[default]
    None,
    Single(Word),
    Array(Array),
}

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    name: String,
    index: Option<Subscript>,
    value: ParsedDataType,
    evaluated_value: DataType,
    append: bool,
}

fn readonly_error(name: &str, core: &mut ShellCore) -> bool {
    core.db.set_param2("?", "1");
    let msg = error::readonly(name);
    error::print(&msg, core);
    false
}

fn bad_subscript_error(sub: &str, core: &mut ShellCore) -> bool {
    core.db.set_param2("?", "1");
    let msg = error::bad_array_subscript(&sub);
    error::print(&msg, core);
    false
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore,
                local: bool, env: bool) -> bool {
        self.evaluated_value = match self.value.clone() {
            ParsedDataType::None      => DataType::Single(SingleData::default()),
            ParsedDataType::Single(v) => {
                match self.eval_as_value2(&v, core) {
                    Some(e) => DataType::Single(SingleData::from(&e)),
                    None => DataType::None,
                }
            },
            ParsedDataType::Array(a)  => {
                match self.eval_as_array2(&mut a.clone(), core) {
                    Some(vec) => DataType::Array(ArrayData::from(vec)),
                    None => DataType::None,
                }
            },
        };

        match env {
            false => self.set_to_shell(core, local),
            true  => self.set_to_env(),
        }
    }

    fn set_assoc(&mut self, core: &mut ShellCore, local: bool) -> bool {
        let index = self.get_index(core);
        let result = match (&self.evaluated_value, index, local) {
            (DataType::Single(v), Some(k), false) 
              => core.db.set_assoc_elem(&self.name, &k, &v.data),
            (DataType::Single(v), Some(k), true) 
              => core.db.set_local_assoc_elem(&self.name, &k, &v.data),
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
            (DataType::Single(v), Some(n), true) 
                => core.db.set_local_array_elem(&self.name, &v.data, n),
            (DataType::Single(v), Some(n), false) 
                => core.db.set_array_elem(&self.name, &v.data, n),
            (_, Some(_), _) => false,
            (data, None, true) 
                => core.db.set_local(&self.name, data.clone()),
            (data, None, false) 
                => core.db.set(&self.name, data.clone()),
        };

        match result {
            true  => true,
            false => readonly_error(&self.name, core),
        }
    }
 
    fn set_param2(&mut self, core: &mut ShellCore, local: bool) -> bool {
        let result = match (&self.evaluated_value, local) {
            (data, true) => core.db.set_local(&self.name, data.clone()),
            (data, false) => core.db.set(&self.name, data.clone()),
        };

        match result {
            true  => true,
            false => readonly_error(&self.name, core),
        }
    }

    fn set_to_shell(&mut self, core: &mut ShellCore, local: bool) -> bool {
        match &self.evaluated_value {
            DataType::None => {
                core.db.set_param2("?", "1");
                return false;
            },
            _ => {},
        }

        if core.db.is_assoc(&self.name) {
            self.set_assoc(core, local)
        }else if core.db.is_array(&self.name) {
            self.set_array(core, local)
        }else {
            self.set_param2(core, local)
        }
    }

    pub fn set_to_env(&mut self) -> bool {
        match &self.evaluated_value {
            DataType::Single(v) => env::set_var(&self.name, &v.data),
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

    fn eval_as_value2(&self, w: &Word, core: &mut ShellCore) -> Option<String> {
        let prev = match self.append {
            true  => core.db.get_param(&self.name),
            false => "".to_string(),
        };

        match w.eval_as_value(core) {
            Some(s) => Some((prev + &s).to_string()),
            None    => None,
        }
    }

    fn eval_as_array2(&self, a: &mut Array, core: &mut ShellCore) -> Option<Vec<String>> {
        let prev = match self.append {
            true  => core.db.get_array_all(&self.name),
            false => vec![],
        };

        match a.eval(core) {
            Some(values) => Some([prev, values].concat()),
            None         => None,
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
            ans.value = ParsedDataType::Array(a);
            Some(ans)
        }else if let Some(w) = Word::parse(feeder, core, false) {
            ans.text += &w.text;
            ans.value = ParsedDataType::Single(w);
            Some(ans)
        }else {
            Some(ans)
        }
    }
}
