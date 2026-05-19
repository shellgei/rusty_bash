//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::Word;
use crate::error::parse::ParseError;
use super::BracedParamExtension;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Option<Word>,
    pub length: Option<Word>,
}

impl BracedParamExtension for Substr {
    fn get_text(&self) -> String { self.text.clone() }
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> { Box::new(self.clone()) }
}

impl Substr {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if !feeder.starts_with(":") {
            return None;
        }
        /*
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

        Some(ans)*/
        None
    }
}
