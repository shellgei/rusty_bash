//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command;
use crate::elements::subscript::Subscript;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::word::Word;

#[derive(Debug, Clone, Default)]
pub struct Array {
    pub text: String,
    pub words: Vec<(Option<Subscript>, Word)>,
}

impl Array {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<Vec<String>, ExecError> {
        let mut ans = vec![];

        for (_, w) in &mut self.words {
            let ws = w.eval(core)?;
            ans.extend(ws);
        }

        Ok(ans)
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.starts_with(")") {
            return false;
        }

        let w = match Word::parse(feeder, core, None) {
            Ok(Some(w)) => w,
            _       => return false,
        };
        ans.text += &w.text;
        ans.words.push((None, w));
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Array>, ParseError> {
        if ! feeder.starts_with("(") {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.text = feeder.consume(1);
        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);

            if let Some(s) = Subscript::parse(feeder, core)? {
                if ! feeder.starts_with("=") {
                    feeder.replace(0, &s.text);
                }
            }

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
                return Ok(None);
            }
        }

        Ok(Some(ans))
    }
}
