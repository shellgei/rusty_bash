//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils::error;
use std::env;
use super::array::Array;
use super::subscript::Subscript;
use super::word::Word;

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
    evaluated_string: Option<String>,
    evaluated_array: Option<Vec<String>>,
    append: bool,
}

fn readonly_error(name: &str, core: &mut ShellCore) -> bool {
    core.db.exit_status = 1;
    let msg = error::readonly(name);
    error::print(&msg, core);
    false
}

fn bad_subscript_error(sub: &str, core: &mut ShellCore) -> bool {
    core.db.exit_status = 1;
    let msg = error::bad_array_subscript(&sub);
    error::print(&msg, core);
    false
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore, layer: usize, env: bool) -> bool {
        match self.value.clone() {
            ParsedDataType::None 
            => self.evaluated_string = Some("".to_string()),
            ParsedDataType::Single(v) 
            => if let Some(e) = self.eval_as_value(&v, core) {
                self.evaluated_string = Some(e);
            }
            ParsedDataType::Array(mut a) 
            => if let Some(vec) = self.eval_as_array(&mut a, core) {
                self.evaluated_array = Some(vec.clone());
            }
        };

        let layer = match layer {
            0 => match core.db.get_layer_pos(&self.name) {
                Some(n) => n,
                None => 0,
            },
            n => n,
        };

        match env {
            false => self.set_to_shell(core, layer),
            true  => self.set_to_env(),
        }
    }

    fn set_assoc(&mut self, core: &mut ShellCore, layer: usize) -> bool {
        let index = self.get_index(core);
        let result = match (&self.evaluated_string, index) {
            (Some(v), Some(k)) 
                => core.db.set_layer_assoc_elem(&self.name, &k, &v, layer),
            _   => return bad_subscript_error(&self.text, core),
        };
        if ! result {
            readonly_error(&self.name, core);
            return false;
        }
        true
    }

    fn set_array(&mut self, core: &mut ShellCore, layer: usize) -> bool {
        let index = match self.get_index(core) {
            Some(s) => {
                match s.parse::<usize>() {
                    Ok(n) => Some(n),
                    _ => return bad_subscript_error(&self.text, core),
                }
            },
            None => None,
        };

        match (&self.evaluated_string, index) {
            (Some(v), Some(n)) => {
                return match core.db.set_layer_array_elem(&self.name, &v, layer, n) {
                    true  => true,
                    false => readonly_error(&self.name, core),
                }
            },
            _ => {},
        }

        let result = match (&self.evaluated_array, index) {
            (Some(a), None) => core.db.set_layer_array(&self.name, a.clone(), layer),
            _ => false,
        };

        match result {
            true  => true,
            false => readonly_error(&self.name, core),
        }
    }
 
    fn set_param(&mut self, core: &mut ShellCore, layer: usize) -> bool {
        let (done, result) = match &self.evaluated_string {
            Some(data) => (true, core.db.set_layer_param(&self.name, &data, layer).is_ok()),
            _ => (false, true),
        };

        if ! result {
            return readonly_error(&self.name, core);
        }
        if done {
            return result;
        }

        let result = match &self.evaluated_array {
            Some(data) => core.db.set_layer_array(&self.name, data.to_vec(), layer),
            _ => false,
        };

        if ! result {
            return readonly_error(&self.name, core);
        }
        result
    }

    fn set_to_shell(&mut self, core: &mut ShellCore, layer: usize) -> bool {
        if self.evaluated_string.is_none()
        && self.evaluated_array.is_none() {
            core.db.exit_status = 1;
            return false;
        }

        if ! core.db.has_value(&self.name) {
            if self.index.is_some() {
                return self.set_array(core, layer);
            }
        }

        if core.db.is_assoc(&self.name) {
            self.set_assoc(core, layer)
        }else if core.db.is_array(&self.name) {
            self.set_array(core, layer)
        }else {
            self.set_param(core, layer)
        }
    }

    pub fn set_to_env(&mut self) -> bool {
        match &self.evaluated_string {
            Some(v) => env::set_var(&self.name, &v),
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

    fn eval_as_value(&self, w: &Word, core: &mut ShellCore) -> Option<String> {
        let prev = match self.append {
            true  => core.db.get_param(&self.name),
            false => "".to_string(),
        };

        match w.eval_as_value(core) {
            Some(s) => Some((prev + &s).to_string()),
            None    => None,
        }
    }

    fn eval_as_array(&self, a: &mut Array, core: &mut ShellCore) -> Option<Vec<String>> {
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
