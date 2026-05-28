//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;
use super::BracedParamExtension;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Option<Word>, //本来は計算式を入れられる
    pub length: Option<Word>, //同上
}

impl BracedParamExtension for Substr {
    fn get_text(&self) -> String { self.text.clone() }
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> { Box::new(self.clone()) }
}

impl Substr {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with(":") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        let mode = WordMode::Exclude(vec![":".to_string(), "}".to_string()]);
        ans.offset = match Word::parse(feeder, core, Some(mode))? {
            Some(w) => {
                ans.text += &w.text.clone();
                Some(w)
            },
            None => None,
        };

        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
