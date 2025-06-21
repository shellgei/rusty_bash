//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::Word;
use crate::error::parse::ParseError;

#[derive(Debug, Clone, Default)]
pub struct Value {
    pub text: String,
    pub value: Word,
    pub evaluated_string: Option<String>,
}

impl Value {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        if let Some(mut w) = Word::parse(feeder, core)? {
            ans.text += &w.text;
            ans.value = w;
        }//パース失敗は左辺が空文字と解釈
        Ok(Some(ans))
    }
}
