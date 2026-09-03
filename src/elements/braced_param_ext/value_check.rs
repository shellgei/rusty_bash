//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use super::{BracedParamExtension, ExecError, ParseError, Parameter};

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub text: String,
    pub symbol: String,
    pub pattern: Word,
}

impl BracedParamExtension for ValueCheck {
    fn exec(&mut self, _: &Parameter, text: &str, _: &mut ShellCore)
        -> Result<String, ExecError> {
        Ok(text.to_string())
    }

    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> {
        Box::new(self.clone())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl ValueCheck {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_parameter_check_symbol();
        if len == 0 { 
            return Ok(None);
        }   

        let mut ans = ValueCheck::default();
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();

        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string()]));
        ans.pattern = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &ans.pattern.text.clone();
//        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
