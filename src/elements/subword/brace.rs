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
            _ => {
                if feeder.starts_with(",") || feeder.starts_with("}") {
                    ans.words.push(Word::new());
                    return true;
                }
                false
            },
        }
    } 

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BraceSubword> {
       if ! feeder.starts_with("{") {
            return None;
        }
        feeder.set_backup();
        core.word_nest.push("{".to_string());

        let mut ans = Self::new();
        let mut closed = false;

        ans.text = feeder.consume(1); // {

        while Self::eat_word(feeder, &mut ans, core) {
            if feeder.starts_with(",") {
                ans.text += &feeder.consume(1);
                core.word_nest.push(",".to_string());
                continue;
            }

            if feeder.starts_with("}") {
                ans.text += &feeder.consume(1);
                closed = true;
            }
            break;
        }

        while core.word_nest.last().unwrap() == "," {
            core.word_nest.pop();
        }
        core.word_nest.pop();

        if closed && ans.words.len() >= 1 {
            feeder.pop_backup();
            ans.text.insert(0, '<');
            ans.text.push('>');
            Some(ans)
        }else{
            feeder.rewind();
            None
        }
    }
}
