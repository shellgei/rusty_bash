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
    fn new() -> UnquotedSubword {
        UnquotedSubword {
            text: String::new(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<UnquotedSubword> {
        let mut ans = Self::new();

        let len = feeder.scanner_word(core);
        if len == 0 {
            return None;
        }
 
        ans.text = feeder.consume(len);
        Some(ans)
    }
}
