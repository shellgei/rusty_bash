//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::braced_param_ext::BracedParamExtension;
use crate::elements::substitution::variable::Variable;
use crate::elements::subword;
use crate::elements::subword::{ExecError, WordMode};
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};
use super::Subword;

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    param: Variable,
    extension: Option<Box<dyn BracedParamExtension>>,
    unknown: String,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.param.text.is_empty() || !self.unknown.is_empty() {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }

        self.text = core.db.get_param(&self.param.text)?;
        Ok(())
    }
}

impl BracedParam {
    fn eat_param(&mut self, feeder: &mut Feeder, core: &mut ShellCore) {
        let mut len = feeder.scanner_name(core);
        if len == 0 {
            len = feeder.scanner_uint(core);
        }
        if len == 0 {
            len = feeder.scanner_special_param();
        }
        if len == 0 {
            return;
        }
        self.param.text = feeder.consume(len);
        self.text += &self.param.text;
    }

    fn eat_end(&mut self, feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        if feeder.is_empty() {
            feeder.feed_additional_line(core)?;
        }

        if feeder.starts_with("}") {
            self.text += &feeder.consume(1);
            return Ok(true);
        }

        if let Some(a) = subword::parse(feeder, core,
                             &Some(WordMode::PermitAnyChar))? {
            self.unknown += &a.get_text();
            self.text += &a.get_text();
            return Ok(false);
        }
        Err(ParseError::UnexpectedSymbol(feeder.consume(feeder.len())))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
                 -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("${") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        ans.eat_param(feeder, core);
        while !ans.eat_end(feeder, core)?{}

        Ok(Some(ans))
    }
}
