//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct EscapedChar {
    pub text: String,
}

impl Subword for EscapedChar {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn make_unquoted_string(&mut self) -> String { self.text[1..].to_string() }
    fn get_type(&self) -> SubwordType { SubwordType::EscapedChar  }
}

impl EscapedChar {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let len = feeder.scanner_escaped_char(core);
        match len > 0 {
            true  => Some(EscapedChar{ text: feeder.consume(len) }),
            false => None,
        }
    }
}
