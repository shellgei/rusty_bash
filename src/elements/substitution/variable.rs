//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use super::subscript::Subscript;

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub text: String,
    pub name: String,
    pub index: Option<Subscript>,
    pub lineno: usize,
}

impl Variable {
    pub fn get_index(&mut self, core: &mut ShellCore,
                     right_is_array: bool, append: bool) -> Result<Option<String>, ExecError> {
        if let Some(mut s) = self.index.clone() {
            if s.text.chars().all(|c| " \n\t[]".contains(c)) {
                return Ok(Some("".to_string()));
            }
            let index = s.eval(core, &self.name)?;
            return Ok(Some(index));
        }

        if core.db.is_array(&self.name) && ! append && ! right_is_array {
            Ok(Some("0".to_string()))
        }else{
            Ok(None)
        }
    }

    fn set_value(&mut self, value: &String, core: &mut ShellCore)
    -> Result<(), ExecError> {
        if self.index.is_none() {
            return core.db.set_param(&self.name, value, None);
        }
    
        let index = self.index.clone().unwrap().eval(core, &self.name)?;
        core.db.set_param2(&self.name, &index, value, None)
    }

    pub fn parse_and_set(arg: &str, value: &str, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut f = Feeder::new(arg);
        match Self::parse(&mut f, core)? {
            Some(mut v) => {
                if ! f.is_empty() {
                    return Err(ExecError::InvalidName(arg.to_string()));
                }
                v.set_value(&value.to_string(), core)
            },
            None => Err(ExecError::InvalidName(arg.to_string())),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.lineno = feeder.lineno;

        let name = feeder.consume(len);
        ans.name = name.clone();
        ans.text += &name;

        if let Some(s) = Subscript::parse(feeder, core)? {
            ans.text += &s.text.clone();
            ans.index = Some(s);
        };

        Ok(Some(ans))
    }
}
