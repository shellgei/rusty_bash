//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use std::env;
use std::collections::HashMap;
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

impl ParsedDataType {
    pub fn get_evaluated_text(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        match self {
            Self::None      => Ok("".to_string()),
            Self::Single(s) => Ok(s.eval_as_value(core)?),
            Self::Array(a) => {
                let mut ans = "(".to_string();
                let mut ws = vec![];
                for (_, w) in &a.words {
                    ws.push( w.eval_as_value(core)? );
                }
                ans += &ws.join(" ");
                ans += ")";
                Ok(ans)
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    name: String,
    index: Option<Subscript>,
    value: ParsedDataType,
    evaluated_string: Option<String>,
    evaluated_array: Option<HashMap<String, String>>,
    append: bool,
}

impl Substitution {
    pub fn eval(&mut self, core: &mut ShellCore, layer: Option<usize>, env: bool)
    -> Result<(), ExecError> {
        let result = match self.value.clone() {
            ParsedDataType::Single(v) => self.eval_as_value(&v, core),
            ParsedDataType::Array(mut a) => self.eval_as_array(&mut a, core),
            ParsedDataType::None => {
                self.evaluated_string = Some("".to_string());
                Ok(())
            },
        };

        if result.is_err() {
            core.db.exit_status = 1;
            return result;
        }

        if env {
            return self.set_to_env();
        }

        let ans = self.set_to_shell(core, layer);
        if ! ans.is_ok() {
            core.db.exit_status = 1;
        }
        ans
    }

    pub fn get_string_for_eval(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut splits = self.text.split("=");
        let front = splits.nth(0).unwrap().to_owned() + "=";
        let rear = self.value.get_evaluated_text(core)?;

        Ok(front + &rear)
    }

    fn set_array(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        match self.get_index(core)? {
            None => {
                if let Some(a) = &self.evaluated_array {
                    core.db.init(&self.name, layer);
                    for e in a {
                        core.db.set_param2(&self.name, &e.0, &e.1, Some(layer))?;
                    }
                    return Ok(());
                }
                return Err(ExecError::Other("no array and no index".to_string()));
            },
            Some(index) => {
                if index.is_empty() {
                    return Err(ExecError::Other(format!("{}[]: invalid index", &self.name)));
                }
                if let Some(v) = &self.evaluated_string {
                    return core.db.set_param2(&self.name, &index, &v, Some(layer));
                }
                return Err(ExecError::Other("indexed to non array variable".to_string()));
            },
        }
    }

    fn set_number_param(&mut self, core: &mut ShellCore, layer: usize)
    -> Result<(), ExecError> {
        let s = match &self.evaluated_string {
            Some(s) => s,
            None => return Err(ExecError::OperandExpected("".to_string())),
        };

        let mut feeder = Feeder::new(&s);
        if let Some(mut exp) = ArithmeticExpr::parse(&mut feeder, core, false, "")? {
            if feeder.len() > 0 {
                return Err(ExecError::SyntaxError(feeder.consume(feeder.len())));
            }
            let ans = exp.eval(core)?;
            return core.db.set_param(&self.name, &ans, Some(layer));
        }

        return Err(ExecError::OperandExpected("".to_string()));
    }
 
    fn set_to_shell(&mut self, core: &mut ShellCore, layer: Option<usize>)
    -> Result<(), ExecError> {
        let layer = core.db.get_target_layer(&self.name, layer);

        if core.db.is_single_num(&self.name) {
            return self.set_number_param(core, layer);
        }else if self.evaluated_string.is_some() && self.index.is_none() {
            let data = self.evaluated_string.clone().unwrap();
            return core.db.set_param(&self.name, &data, Some(layer));
        }

        self.set_array(core, layer)
    }

    pub fn set_to_env(&mut self) -> Result<(), ExecError> {
        match &self.evaluated_string {
            Some(v) => env::set_var(&self.name, &v),
            _ => return Err(ExecError::Other(format!("{}: invalid environmental variable", &self.name))),
        }
        Ok(())
    }

    pub fn get_index(&mut self, core: &mut ShellCore) -> Result<Option<String>, ExecError> {
        match self.index.clone() {
            Some(mut s) => {
                if s.text.chars().all(|c| " \n\t[]".contains(c)) {
                    return Ok(Some("".to_string()));
                }
                let index = s.eval(core, &self.name)?;
                Ok(Some(index)) 
            },
            None => {
                match core.db.is_array(&self.name)
                    && ! self.append
                    && self.evaluated_array == None {
                    true  => Ok(Some("0".to_string())),
                    false => Ok(None),
                }
            },
        }
    }

    fn eval_as_value(&mut self, w: &Word, core: &mut ShellCore) -> Result<(), ExecError> {
        let prev = match self.append {
            true  => core.db.get_param(&self.name).unwrap_or(String::new()),
            false => "".to_string(),
        };

        let s = w.eval_as_value(core)?;
        self.evaluated_string = Some(prev + &s);
        Ok(())
    }

    fn eval_as_array(&mut self, a: &mut Array, core: &mut ShellCore) -> Result<(), ExecError> {
        let prev = match self.append {
            true  => core.db.get_array_all(&self.name),
            false => vec![],
        };

        let mut i = 0;
        let mut hash = HashMap::new();
        for e in prev {
            hash.insert(i.to_string(), e);
            i += 1;
        }

        let values = a.eval(core)?;
        for (s, v) in values {
            match s {
                Some(mut sub) => {
                    let index = sub.eval(core, &self.name)?;
                    hash.insert(index, v)
                },
                None => hash.insert(i.to_string(), v),
            };
            i += 1;
        }
        self.evaluated_array = Some(hash);
        Ok(())
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

        if let Some(s) = Subscript::parse(feeder, core)? {
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

        if let Some(a) = Array::parse(feeder, core)? {
            ans.text += &a.text;
            ans.value = ParsedDataType::Array(a);
        }else if let Ok(Some(w)) = Word::parse(feeder, core, None) {
            ans.text += &w.text;
            ans.value = ParsedDataType::Single(w);
        }
        Ok(Some(ans))
    }
}
