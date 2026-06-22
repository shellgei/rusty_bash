//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::BracedExcludeension;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::elements::parameter::Parameter;
//use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: ArithmeticExpr,
    pub length: Option<ArithmeticExpr>,
    unknown: String,
}

impl BracedExcludeension for Substr {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(
        &mut self,
        v: &Parameter,
        text: &str,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        if core.db.exist(&v.name) {
            self.get(text, core)
        } else {
            Ok(text.to_string())
        }
    }

    fn boxed_clone(&self) -> Box<dyn BracedExcludeension> {
        Box::new(self.clone())
    }
    fn has_array_replace(&self) -> bool {
        true
    }

    fn init_array(
        &mut self,
        param: &Parameter,
        array: &mut Vec<String>,
        text: &mut String,
        core: &mut ShellCore,
    ) -> Result<(), ExecError> {
        let ifs = core.db.get_ifs_head();
        match param.name.as_str() {
            "@" => self.set_partial_position_params(array, text, core, " "),
            "*" => self.set_partial_position_params(array, text, core, &ifs),
            _ => self.set_partial_array(&param.name, array, text, core),
        }
    }

    fn receive_unknown(&mut self, unknown: &mut String) {
        self.unknown = unknown.clone();
        unknown.clear();
    }
}

impl Substr {
    fn set_partial_position_params(
        &mut self,
        array: &mut Vec<String>,
        text: &mut String,
        core: &mut ShellCore,
        ifs: &str,
    ) -> Result<(), ExecError> {
        if self.offset.text.is_empty() && self.length.is_none() {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }

        *array = core.db.get_vec("@", false)?;
        let mut n = self.offset.eval_as_int(core)?;
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
            *text = array.join(ifs);
            return Ok(());
        }

        let mut length = match self.length.clone() {
            None => return Err(ExecError::BadSubstitution(self.text.clone())),
            Some(ofs) => ofs,
        };

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
        if self.offset.text.is_empty() && self.length.is_none() {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }

        let mut n = self.offset.eval_as_int(core)?;
        let len = core.db.index_based_len(name);
        if n < 0 {
            n += len as i128;
            if n < 0 {
                *text = "".to_string();
                *array = vec![];
                return Ok(());
            }
        }

        *array = core.db.get_vec_from(name, n as usize, true)?;

        if self.length.is_none() {
            *text = array.join(" ");
            return Ok(());
        }

        let mut length = match self.length.clone() {
            None => return Err(ExecError::BadSubstitution(self.text.clone())),
            Some(ofs) => ofs,
        };

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
        if self.offset.text.is_empty() && self.length.is_none() {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }

        let mut ans: String;
        let mut n = self.offset.eval_as_int(core)?;
        let len = text.chars().count();

        if n < 0 {
            n += len as i128;
            if n < 0 {
                return Ok("".to_string());
            }
        }

        ans = text
            .chars()
            .enumerate()
            .filter(|(i, _)| (*i as i128) >= n)
            .map(|(_, c)| c)
            .collect();

        if ans.is_empty() {
            return Ok(ans);
        }

        if !self.unknown.is_empty() {
            return Err(ParseError::UnexpectedSymbol(self.unknown.clone()).into());
        }

        if self.length.is_some() {
            ans = self.length(&ans, core)?;
        }

        Ok(ans)
    }

    fn length(&mut self, text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut n = self.length.as_mut().unwrap().eval_as_int(core)?;

        if n < 0 {
            let str_len = text.chars().count();
            n += str_len as i128;
        }
        //dbg!("{:?}", &n);
        Ok(text
            .chars()
            .enumerate()
            .filter(|(i, _)| (*i as i128) < n)
            .map(|(_, c)| c)
            .collect())
    }

    fn eat_length(&mut self, feeder: &mut Feeder, core: &mut ShellCore) -> Result<(), ParseError> {
        if !feeder.starts_with(":") {
            return Ok(());
        }
        self.text += &feeder.consume(1);

        self.length = ArithmeticExpr::parse(feeder, core, true, ":")?;
        if let Some(ref a) = self.length {
            self.text += &a.text.clone();
        }

        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with(":") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        ans.offset =
            ArithmeticExpr::parse(feeder, core, true, ":")?.unwrap_or(ArithmeticExpr::new());
        ans.text += &ans.offset.text.clone();
        ans.eat_length(feeder, core)?;
        Ok(Some(ans))
    }
}
