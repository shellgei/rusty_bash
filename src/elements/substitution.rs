//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::{ExecError, ParseError};
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

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore, layer: Option<usize>, env: bool) -> Result<(), ExecError> {
        match self.value.clone() {
            ParsedDataType::None 
            => self.evaluated_string = Some("".to_string()),
            ParsedDataType::Single(v) 
            => if let Some(e) = self.eval_as_value(&v, core) {
                self.evaluated_string = Some(e);
            }
            ParsedDataType::Array(mut a) 
            => if let Ok(vec) = self.eval_as_array(&mut a, core) {
                self.evaluated_array = Some(vec.clone());
            }
        };

        match env {
            false => {
                let ans = self.set_to_shell(core, layer);
                if ! ans.is_ok() {
                    core.db.exit_status = 1;
                }
                ans
            },
            true  => self.set_to_env(),
        }
    }

    fn set_assoc(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        let index = self.get_index(core);
        match (&self.evaluated_string, index) {
            (Some(v), Ok(k)) 
                => core.db.set_assoc_elem(&self.name, &k, &v, Some(layer)),
            _   => Err(ExecError::Other("evaluation error 1".to_string())),
        }
    }

    fn set_array(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        if ! self.get_index(core).is_ok() {
            return match &self.evaluated_array {
                Some(a) => core.db.set_array(&self.name, a.clone(), Some(layer)),
                _ => Err(ExecError::Other("no array and no index".to_string())),
            };
        }

        let index = self.get_index(core)
                        .unwrap()
                        .parse::<usize>().map_err(|e| ExecError::Other(format!("{:?}", e)))?;

        if let Some(v) = &self.evaluated_string {
            return core.db.set_array_elem(&self.name, &v, index, Some(layer));
        }
        Err(ExecError::Other("indexed to non array variable".to_string()))
    }
 
    fn set_param(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        let (done, result) = match &self.evaluated_string {
            Some(data) => (true, core.db.set_param(&self.name, &data, Some(layer))),
            _ => (false, Ok(()) ),
        };

        if result.is_err() || done {
            return result;
        }

        match &self.evaluated_array {
            Some(data) => core.db.set_array(&self.name, data.to_vec(), Some(layer)),
            _ => Err(ExecError::Other("evaluation error 3".to_string())),
        }
    }

    fn set_to_shell(&mut self, core: &mut ShellCore, layer: Option<usize>) -> Result<(), ExecError> {
        let layer = core.db.get_target_layer(&self.name, layer);

        if self.evaluated_string.is_none()
        && self.evaluated_array.is_none() {
            core.db.exit_status = 1;
            return Err(ExecError::Other("no value".to_string()));
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

    pub fn set_to_env(&mut self) -> Result<(), ExecError> {
        match &self.evaluated_string {
            Some(v) => env::set_var(&self.name, &v),
            _ => return Err(ExecError::Other(format!("{}: invalid environmental variable", &self.name))),
        }
        Ok(())
    }

    pub fn get_index(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        match self.index.clone() {
            Some(mut s) => {
                if s.text.chars().all(|c| " \n\t[]".contains(c)) {
                    return Ok("".to_string());
                }
                s.eval(core, &self.name)
            },
            _ => Err(ExecError::ArrayIndexInvalid("".to_string())),
        }
    }

    fn eval_as_value(&self, w: &Word, core: &mut ShellCore) -> Option<String> {
        let prev = match self.append {
            true  => core.db.get_param(&self.name).unwrap_or(String::new()),
            false => "".to_string(),
        };

        match w.eval_as_value(core) {
            Some(s) => Some((prev + &s).to_string()),
            None    => None,
        }
    }

    fn eval_as_array(&self, a: &mut Array, core: &mut ShellCore) -> Result<Vec<String>, String> {
        let prev = match self.append {
            true  => core.db.get_array_all(&self.name),
            false => vec![],
        };

        let values = a.eval(core)?;
        Ok([prev, values].concat())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return Ok(None);
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
            return Ok(None);
        }
        feeder.pop_backup();

        if let Some(a) = Array::parse(feeder, core) {
            ans.text += &a.text;
            ans.value = ParsedDataType::Array(a);
        }else if let Ok(Some(w)) = Word::parse(feeder, core, false) {
            ans.text += &w.text;
            ans.value = ParsedDataType::Single(w);
        }
        Ok(Some(ans))
    }
}
