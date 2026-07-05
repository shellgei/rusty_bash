//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;
use super::BracedParamExtension;
use crate::elements::parameter::Parameter;
use crate::error::exec::ExecError;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Word, //本来は計算式を入れられる
    pub length: Option<Word>, //同上
}

impl BracedParamExtension for Substr {
    fn exec(&mut self, v: &Parameter, text: &str, core: &mut ShellCore)
    -> Result<String, ExecError> {
        Ok(text.to_string())
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> { Box::new(self.clone()) }
}

impl Substr {
    fn eat_length(&mut self, feeder: &mut Feeder,
                  core: &mut ShellCore) -> Result<(), ParseError> {
        if !feeder.starts_with(":") {
            return Ok(());
        }
        self.text += &feeder.consume(1);

        let mode = WordMode::PermitAnyUntil(vec!["}".to_string()]);
        self.length = Some( Word::parse(feeder, core, Some(mode))?.unwrap_or(Word::default()) );
        self.text += &self.length.as_mut().unwrap().text.clone();

        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with(":") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        let mode = WordMode::PermitAnyUntil(vec![":".to_string(), "}".to_string()]);
        ans.offset = Word::parse(feeder, core, Some(mode))?.unwrap_or(Word::default());
        ans.text += &ans.offset.text.clone();
        ans.eat_length(feeder, core)?;

//        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
