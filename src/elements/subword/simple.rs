//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use super::Subword;

#[derive(Debug, Clone)]
pub struct SimpleSubword {
    pub text: String,
}

impl Subword for SimpleSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
}

impl SimpleSubword {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let len = feeder.scanner_subword_symbol();
        if len > 0 {
            return Some( Self{ text :feeder.consume(len) } );
        }

        let len = feeder.scanner_subword(core);
        if len > 0 {
            return Some( Self{ text :feeder.consume(len) } );
        }

        None
    }
}
