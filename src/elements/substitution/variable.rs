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
    pub fn eval(&mut self, _: &mut ShellCore) -> Result<(), ExecError> {
        Ok(())
    }

    pub fn get_index(&mut self, core: &mut ShellCore,
                     right_is_array: bool, append: bool) -> Result<Option<String>, ExecError> {
        match self.index.clone() {
            Some(mut s) => {
                if s.text.chars().all(|c| " \n\t[]".contains(c)) {
                    return Ok(Some("".to_string()));
                }
                let index = s.eval(core, &self.name)?;
                Ok(Some(index)) 
            },
            None => {
                if core.db.is_array(&self.name) && ! append && ! right_is_array {
                    Ok(Some("0".to_string()))
                }else{
                    Ok(None)
                }
            },
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
