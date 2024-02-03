//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct UnquotedSubword {
    pub text: String,
}

impl Subword for UnquotedSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
}

impl UnquotedSubword {
    fn new(s: &str) -> UnquotedSubword {
        UnquotedSubword {
            text: s.to_string(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<UnquotedSubword> {
        let len = feeder.scanner_word(core);
        if len == 0 {
            None
        }else{
            Some(Self::new( &feeder.consume(len) ))
        }
    }
}

