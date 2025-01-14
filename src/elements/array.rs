//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command;
use super::word::Word;

#[derive(Debug, Clone, Default)]
pub struct Array {
    pub text: String,
    pub words: Vec<Word>,
}

impl Array {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<Vec<String>, String> {
        let mut ans = vec![];

        for w in &mut self.words {
            let ws = w.eval(core)?;
            ans.extend(ws);
            /*
            match w.eval(core) {
                None     => return Err("evaluation error".to_string()),
                Some(ws) => ans.extend(ws),
            }*/
        }

        Ok(ans)
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.starts_with(")") {
            return false;
        }

        let w = match Word::parse(feeder, core, false) {
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

        let mut ans = Self::default();
        ans.text = feeder.consume(1);
        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if Self::eat_word(feeder, &mut ans, core) {
                continue;
            }

            if feeder.starts_with(")") {
                ans.text += &feeder.consume(1);
                break;
            }else if feeder.starts_with("\n") {
                ans.text += &feeder.consume(1);
            }

            if feeder.len() != 0 || ! feeder.feed_additional_line(core).is_ok() {
                return None;
            }
        }

        Some(ans)
    }
}
