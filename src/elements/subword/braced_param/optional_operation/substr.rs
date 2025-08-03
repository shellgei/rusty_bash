//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::super::Variable;
use super::OptionalOperation;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Option<ArithmeticExpr>,
    pub length: Option<ArithmeticExpr>,
}

impl OptionalOperation for Substr {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(
        &mut self,
        _: &Variable,
        text: &str,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        self.get(text, core)
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {
        Box::new(self.clone())
    }
    fn has_array_replace(&self) -> bool {
        true
    }

    fn set_array(
        &mut self,
        param: &Variable,
        array: &mut Vec<String>,
        text: &mut String,
        core: &mut ShellCore,
    ) -> Result<(), ExecError> {
        match param.name.as_str() {
            "@" | "*" => self.set_partial_position_params(array, text, core),
            _ => self.set_partial_array(&param.name, array, text, core),
        }
    }
}

impl Substr {
    fn set_partial_position_params(
        &mut self,
        array: &mut Vec<String>,
        text: &mut String,
        core: &mut ShellCore,
    ) -> Result<(), ExecError> {
        let offset = self.offset.as_mut().unwrap();

        if offset.text.is_empty() {
            return Err(ExecError::BadSubstitution(String::new()));
        }

        *array = core.db.get_vec("@", false)?;
        let mut n = offset.eval_as_int(core)?;
        let len = array.len();

        if n < 0 {
            n += len as i128;
            if n < 0 {
                *text = "".to_string();
                *array = vec![];
                return Ok(());
            }
        }

        let mut start = std::cmp::max(0, n) as usize;
        start = std::cmp::min(start, array.len());
        *array = array.split_off(start);

        if self.length.is_none() {
            *text = array.join(" ");
            return Ok(());
        }

        let mut length = match self.length.clone() {
            None => return Err(ExecError::BadSubstitution("".to_string())),
            Some(ofs) => ofs,
        };

        if length.text.is_empty() {
            return Err(ExecError::BadSubstitution("".to_string()));
        }

        let n = length.eval_as_int(core)?;
        if n < 0 {
            return Err(ExecError::SubstringMinus(n));
        }
        let len = std::cmp::min(n as usize, array.len());
        let _ = array.split_off(len);

        *text = array.join(" ");
        Ok(())
    }

    fn set_partial_array(
        &mut self,
        name: &str,
        array: &mut Vec<String>,
        text: &mut String,
        core: &mut ShellCore,
    ) -> Result<(), ExecError> {
        let offset = self.offset.as_mut().unwrap();

        if offset.text.is_empty() {
            return Err(ExecError::BadSubstitution(String::new()));
        }

        let mut n = offset.eval_as_int(core)?;
        let len = core.db.index_based_len(name);
        if n < 0 {
            n += len as i128;
            if n < 0 {
                *text = "".to_string();
                *array = vec![];
                return Ok(());
            }
        }

        //let start = std::cmp::max(0, n) as usize;
        //*array = core.db.get_vec_from(name, start, true)?;
        *array = core.db.get_vec_from(name, n as usize, true)?;

        if self.length.is_none() {
            *text = array.join(" ");
            return Ok(());
        }

        let mut length = match self.length.clone() {
            None => return Err(ExecError::BadSubstitution("".to_string())),
            Some(ofs) => ofs,
        };

        if length.text.is_empty() {
            return Err(ExecError::BadSubstitution("".to_string()));
        }

        let n = length.eval_as_int(core)?;
        if n < 0 {
            return Err(ExecError::SubstringMinus(n));
        }
        let len = std::cmp::min(n as usize, array.len());
        let _ = array.split_off(len);

        *text = array.join(" ");
        Ok(())
    }

    pub fn get(&mut self, text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
        let offset = self.offset.as_mut().unwrap();

        if offset.text.is_empty() {
            let err = ArithError::OperandExpected("".to_string());
            return Err(ExecError::ArithError("".to_string(), err));
        }

        let mut n = offset.eval_as_int(core)?;
        let len = text.chars().count();

        if n < 0 {
            n += len as i128;
            if n < 0 {
                return Ok("".to_string());
            }
        }

        let mut ans: String = text
            .chars()
            .enumerate()
            .filter(|(i, _)| (*i as i128) >= n)
            .map(|(_, c)| c)
            .collect();

        if self.length.is_some() {
            ans = self.length(&ans, core)?;
        }

        Ok(ans)
    }

    fn length(&mut self, text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
        let n = self.length.as_mut().unwrap().eval_as_int(core)?;
        Ok(text
            .chars()
            .enumerate()
            .filter(|(i, _)| (*i as i128) < n)
            .map(|(_, c)| c)
            .collect())
    }

    fn eat_length(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        if !feeder.starts_with(":") {
            return;
        }
        ans.text += &feeder.consume(1);
        ans.length = match ArithmeticExpr::parse(feeder, core, true, ":") {
            Ok(Some(a)) => {
                ans.text += &a.text.clone();
                Some(a)
            }
            _ => None,
        };
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if !feeder.starts_with(":") {
            return None;
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        ans.offset = match ArithmeticExpr::parse(feeder, core, true, ":") {
            Ok(Some(a)) => {
                ans.text += &a.text.clone();
                Self::eat_length(feeder, &mut ans, core);
                Some(a)
            }
            _ => None,
        };

        Some(ans)
    }
}
