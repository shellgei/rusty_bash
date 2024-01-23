//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;
use crate::elements::word::Word;

#[derive(Debug)]
pub struct BraceSubword {
    text: String,
    words: Vec<Word>,
}

impl Subword for BraceSubword {
    fn get_text(&self) -> String { self.text.clone() }
}

impl BraceSubword {
    fn new() -> BraceSubword {
        BraceSubword {
            text: String::new(),
            words: vec![],
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut BraceSubword, core: &mut ShellCore) -> bool {
        match Word::parse(feeder, core) {
            Some(w) => {
                ans.text += &w.text;
                ans.words.push(w);
                true
            },
            _  => false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BraceSubword> {
        let mut ans = Self::new();

        let len = feeder.scanner_word(core);
        if len == 0 {
            return None;
        }
 
        ans.text = feeder.consume(len);
        Some(ans)
    }
}
