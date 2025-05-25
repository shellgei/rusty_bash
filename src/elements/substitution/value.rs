//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::word::WordMode;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use std::collections::HashMap;
use super::array::Array;
use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub enum ParsedDataType {
    #[default]
    None,
    Single(Word),
    Array(Array),
}

impl ParsedDataType {
    pub fn get_evaluated_text(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        match self {
            Self::None      => Ok("".to_string()),
            Self::Single(s) => Ok(s.eval_as_value(core)?),
            Self::Array(a) => {
                let mut ans = "(".to_string();
                let mut ws = vec![];
                for (_, w) in &a.words {
                    ws.push( w.eval_as_value(core)? );
                }
                ans += &ws.join(" ");
                ans += ")";
                Ok(ans)
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Value {
    pub text: String,
    pub value: ParsedDataType,
    pub evaluated_string: Option<String>,
    pub evaluated_array: Option<HashMap<String, String>>,
}

impl Value {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        if let Some(a) = Array::parse(feeder, core)? {
            ans.text += &a.text;
            ans.value = ParsedDataType::Array(a);
        }else if let Ok(Some(mut w)) = Word::parse(feeder, core, None) {
            w.mode = Some(WordMode::RightOfSubstitution);
            ans.text += &w.text;
            ans.value = ParsedDataType::Single(w);
        }
        Ok(Some(ans))
    }
}
