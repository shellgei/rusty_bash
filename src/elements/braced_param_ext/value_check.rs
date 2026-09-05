//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use super::{BracedParamExtension, ExecError, ParseError, Parameter};

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub text: String,
    pub symbol: String,
    pub alter: Word,
}

impl BracedParamExtension for ValueCheck {
    fn exec(&mut self, v: &Parameter, text: &str, core: &mut ShellCore)
        -> Result<String, ExecError> {
        let mut check_ok = match self.symbol.starts_with(":") {
            true  => !text.is_empty(),
            false => core.db.exist(&v.text),
        };

        if self.symbol.ends_with("+") {
            check_ok = !check_ok;
        }
        println!("{:?}", if check_ok { "チェックOK" } else {"処理が必要"} );
        if check_ok {
            return Ok(text.to_string());
        }

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

        let mut ans = Self::default();
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();

        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string()]));
        ans.alter = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &ans.alter.text.clone();
//        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
