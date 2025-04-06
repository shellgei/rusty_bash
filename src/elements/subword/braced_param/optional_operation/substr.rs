//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use super::super::Param;
use super::OptionalOperation;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Option<ArithmeticExpr>,
    pub length: Option<ArithmeticExpr>,
}

impl OptionalOperation for Substr {
    fn get_text(&self) -> String {self.text.clone()}
    fn exec(&mut self, _: &Param, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        self.get(text, core)
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {Box::new(self.clone())}
    fn is_substr(&self) -> bool {true}

    fn set_array(&mut self, param: &Param, array: &mut Vec<String>,
                    text: &mut String, core: &mut ShellCore) -> Result<(), ExecError> {
        match param.name.as_str() {
            "@" => self.set_partial_position_params(array, text, core),
            _   => self.set_partial_array(&param.name, array, text, core),
        }
    }
}

impl Substr {
    fn set_partial_position_params(&mut self, array: &mut Vec<String>,
                    text: &mut String, core: &mut ShellCore) -> Result<(), ExecError> {
        let offset = self.offset.as_mut().unwrap();
    
        if offset.text == "" {
            return Err(ExecError::BadSubstitution(String::new()));
        }
    
        *array = core.db.get_array_all("@");
        match offset.eval_as_int(core) {
            None => return Err(ExecError::OperandExpected(offset.text.clone())),
            Some(n) => {
                let mut start = std::cmp::max(0, n) as usize;
                start = std::cmp::min(start, array.len()) as usize;
                *array = array.split_off(start);
            },
        };
    
        if self.length.is_none() {
            *text = array.join(" ");
            return Ok(());
        }

    
        let mut length = match self.length.clone() {
            None => return Err(ExecError::BadSubstitution("".to_string())),
            Some(ofs) => ofs,
        };
    
        if length.text == "" {
            return Err(ExecError::BadSubstitution("".to_string()));
        }
    
        match length.eval_as_int(core) {
            None => return Err(ExecError::BadSubstitution(length.text.clone())),
            Some(n) => {
                if n < 0 {
                    return Err(ExecError::SubstringMinus(n));
                }
                let len = std::cmp::min(n as usize, array.len());
                let _ = array.split_off(len);
            },
        };
    
        *text = array.join(" ");
        Ok(())
    }

    fn set_partial_array(&mut self, name: &str, array: &mut Vec<String>,
                    text: &mut String, core: &mut ShellCore) -> Result<(), ExecError> {
        let offset = self.offset.as_mut().unwrap();
    
        if offset.text == "" {
            return Err(ExecError::BadSubstitution(String::new()));
        }
    
        *array = core.db.get_array_all(name);
        match offset.eval_as_int(core) {
            None => return Err(ExecError::OperandExpected(offset.text.clone())),
            Some(n) => {
                let mut start = std::cmp::max(0, n) as usize;
                start = std::cmp::min(start, array.len()) as usize;
                *array = array.split_off(start);
            },
        };
    
        if self.length.is_none() {
            *text = array.join(" ");
            return Ok(());
        }
    
        let mut length = match self.length.clone() {
            None => return Err(ExecError::BadSubstitution("".to_string())),
            Some(ofs) => ofs,
        };
    
        if length.text == "" {
            return Err(ExecError::BadSubstitution("".to_string()));
        }
    
        match length.eval_as_int(core) {
            None => return Err(ExecError::BadSubstitution(length.text.clone())),
            Some(n) => {
                if n < 0 {
                    return Err(ExecError::SubstringMinus(n));
                }
                let len = std::cmp::min(n as usize, array.len());
                let _ = array.split_off(len);
            },
        };
    
        *text = array.join(" ");
        Ok(())
    }

    pub fn get(&mut self, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        let offset = self.offset.as_mut().unwrap();
    
        if offset.text == "" {
            return Err(ExecError::OperandExpected("".to_string()));
        }
    
        let mut ans;
        match offset.eval_as_int(core) {
            None => return Err(ExecError::OperandExpected(offset.text.clone())),
            Some(n) => {
                ans = text.chars().enumerate()
                      .filter(|(i, _)| (*i as i128) >= n)
                      .map(|(_, c)| c).collect();
            },
        };
    
        if self.length.is_some() {
            ans = self.length(&ans, core)?;
        }
    
        Ok(ans)
    }
    
    fn length(&mut self, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        match self.length.as_mut().unwrap().eval_as_int(core) {
            Some(n) => Ok(text.chars().enumerate()
                            .filter(|(i, _)| (*i as i128) < n)
                            .map(|(_, c)| c).collect()),
            None => return Err(ExecError::OperandExpected(self.length.clone().unwrap().text.clone())),
        }
    }

    fn eat_length(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        if ! feeder.starts_with(":") {
            return;
        }
        ans.text += &feeder.consume(1);
        ans.length = match ArithmeticExpr::parse(feeder, core, true, ":") {
            Ok(Some(a)) => {
                ans.text += &a.text.clone();
                Some(a)
            },
            _ => None,
        };
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with(":") {
            return None;
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        ans.offset = match ArithmeticExpr::parse(feeder, core, true, ":") {
            Ok(Some(a)) => {
                ans.text += &a.text.clone();
                Self::eat_length(feeder, &mut ans, core);
                Some(a)
            },
            _ => None,
        };

        Some(ans)
    }
}
