//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;

#[derive(Debug)]
pub struct UnquotedSubword {
    text: String,
}

impl Subword for UnquotedSubword {
    fn get_text(&self) -> String { self.text.clone() }
}

impl UnquotedSubword {
    fn new(s: &str) -> UnquotedSubword {
        UnquotedSubword {
            text: s.to_string(),
        }
    } 

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<UnquotedSubword> {
        if feeder.starts_with("{") 
        || feeder.starts_with(",") 
        || feeder.starts_with("}") {
            return Some(Self::new( &feeder.consume(1) )); 
        }

        let len = feeder.scanner_word(core);
        if len == 0 {
            return None;
        }
 
        Some( Self::new(&feeder.consume(len)) )
    }
}
