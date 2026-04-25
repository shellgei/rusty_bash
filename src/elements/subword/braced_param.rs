//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::substitution::variable::Variable;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};
use super::Subword;

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    param: Variable,
    unknown: String,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }
}

impl BracedParam {
    fn eat_param(&mut self, feeder: &mut Feeder, core: &mut ShellCore) {
        let len = feeder.scanner_name(core);
        if len != 0 {
            self.param = Variable::default();
            self.param.text = feeder.consume(len);
            self.text += &self.param.text;
        }
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

        let len = match feeder.starts_with("\\}")
                  || feeder.starts_with("\\\\") {
            true => 2,
            false => feeder.scanner_char(),
        };  

        let unknown = feeder.consume(len);
        self.unknown += &unknown.clone();
        self.text += &unknown;
        Ok(false)
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

        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
