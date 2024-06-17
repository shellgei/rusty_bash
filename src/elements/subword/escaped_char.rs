//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct EscapedChar {
    pub text: String,
}

impl Subword for EscapedChar {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
}

impl EscapedChar {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        match feeder.scanner_escaped_char(core) {
            0 => None,
            n => Some(EscapedChar{ text: feeder.consume(n) }),
        }
    }
}
