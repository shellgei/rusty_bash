//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;

#[derive(Debug, Clone, Default)]
pub struct Value {
    pub text: String,
    pub value: Word,
    pub evaluated_string: String,
}

impl Value {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        self.evaluated_string = self.value.eval_as_value(core)?;
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder,
        core: &mut ShellCore, permit_space: bool)
    -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        let mode = match permit_space {
            true  => Some(WordMode::PermitAnyChar),
            false => None,
        };

        if let Some(w) = Word::parse(feeder, core, None)? {
            ans.text += &w.text;
            ans.value = w;
        }//パース失敗は左辺が空文字と解釈
        Ok(Some(ans))
    }
}
