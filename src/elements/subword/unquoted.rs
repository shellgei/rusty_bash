//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};

#[derive(Debug)]
pub struct UnquotedSubword {
    pub text: String,
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
