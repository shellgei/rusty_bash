//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use crate::elements::substitution::variable::Variable;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::BracedParamExtension;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Word, //本来は計算式を入れられる
    pub length: Option<Word>, //同上
}

impl BracedParamExtension for Substr {
    fn get_text(&self) -> String { self.text.clone() }

    fn exec(&mut self, _: &Variable, text: &str,
            core: &mut ShellCore) -> Result<String, ExecError> {
        if self.offset.text.is_empty() && self.length.is_none() {
            return Err(ExecError::BadSubstitution(self.text.clone()));
        }

        let mut n = match self.offset.eval_as_value(core)?.parse::<i32>() {
            Ok(num) => num,
            _ => return Err(ExecError::BadSubstitution(self.text.clone())),
        };
        let len = text.chars().count() as i32;

        if n < 0 { 
            n += len;
            if n < 0 { 
                return Ok("".to_string());
            }
        }

        let mut ans = text.chars().enumerate()
            .filter(|(i, _)| (*i as i32) >= n)
            .map(|(_, c)| c).collect::<String>();

        Ok(ans)
    }

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
                w
            },
            None => Word::default(),
        };

        //dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
