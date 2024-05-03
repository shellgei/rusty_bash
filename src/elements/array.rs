//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command;
use super::subword;
use super::subword::Subword;
use super::word::Word;

#[derive(Debug, Clone)]
pub struct Array {
    pub text: String,
    pub words: Vec<Word>,
}

impl Array {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<Vec<String>> {
        None
    }

    pub fn new() -> Array {
        Array {
            text: String::new(),
            words: vec![],
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        command::eat_blank_with_comment(feeder, core, &mut ans.text);
        if feeder.starts_with(")") {
            return false;
        }

        let w = match Word::parse(feeder, core) {
            Some(w) => w,
            _       => return false,
        };
        ans.text += &w.text;
        ans.words.push(w);
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Array> {
        if ! feeder.starts_with("(") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(1);
        while Self::eat_word(feeder, &mut ans, core) {}

        if feeder.starts_with(")") {
            ans.text += &feeder.consume(1);
            dbg!("{:?}", &ans);
            Some(ans)
        }else {
            None
        }
    }
}
