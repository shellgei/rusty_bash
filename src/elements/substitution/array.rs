//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command;
use super::subscript::Subscript;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub struct Array {
    pub text: String,
    pub words: Vec<(Option<Subscript>, Word)>,
    error_strings: Vec<String>,
}

impl Array {
    pub fn eval(&mut self, core: &mut ShellCore, as_int: bool, as_assoc: bool)
    -> Result<Vec<(Option<Subscript>, String)>, ExecError> {
        if let Some(c) = self.error_strings.last() {
            return Err(ExecError::SyntaxError(c.to_string()));
        }

        let mut ans = vec![];

        if as_int {
            for (s, w) in &mut self.words {
                ans.push( (s.clone(), w.eval_as_integer(core)?) );
            }
        }else{
            for (s, w) in &mut self.words {
                if as_assoc {
                    ans.push( (s.clone(), w.eval_as_value(core)?) );
                }else{
                    for e in w.eval(core)? {
                        ans.push( (s.clone(), e) );
                    }
                }
            }
        }
        Ok(ans)
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self,
                sub: Option<Subscript>, core: &mut ShellCore) -> bool {
        if feeder.starts_with(")") {
            return false;
        }

        let w = match Word::parse(feeder, core, None) {
            Ok(Some(w)) => w,
            _       => return false,
        };
        ans.text += &w.text;
        ans.words.push((sub, w));
        true
    }

    fn eat_subscript(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Self)
    -> Result<Option<Subscript>, ParseError> {
        if let Some(s) = Subscript::parse(feeder, core)? {
            if feeder.starts_with("=") {
                ans.text += &s.text.clone();
                ans.text += &feeder.consume(1);
                return Ok(Some(s));
            }else{
                feeder.replace(0, &s.text);
            }
        }
        Ok(None)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Array>, ParseError> {
        if ! feeder.starts_with("(") {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.text = feeder.consume(1);
        loop {
            command::eat_blank_lines(feeder, core, &mut ans.text)?;

            let sub = Self::eat_subscript(feeder, core, &mut ans)?;
            if Self::eat_word(feeder, &mut ans, sub, core) {
                continue;
            }

            if feeder.len() != 0 {
                if feeder.starts_with(")") {
                    ans.text += &feeder.consume(1);
                    break;
                }else if feeder.starts_with("\n") {
                    ans.text += &feeder.consume(1);
                }

                let len = feeder.scanner_char();
                ans.error_strings.push(feeder.consume(len));
                continue;

            }else if ! feeder.feed_additional_line(core).is_ok() {
                return Ok(None);
            }
        }

        Ok(Some(ans))
    }
}
