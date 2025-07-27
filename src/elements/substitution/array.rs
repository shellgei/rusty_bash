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
    pub words: Vec<(Option<Subscript>, bool, Word)>, //bool: true if append
    error_strings: Vec<String>,
}

impl Array {
    pub fn eval(&mut self, core: &mut ShellCore, as_int: bool, as_assoc: bool)
    -> Result<Vec<(Option<Subscript>, bool, String)>, ExecError> {
        if let Some(c) = self.error_strings.last() {
            return Err(ExecError::SyntaxError(c.to_string()));
        }

        let mut ans = vec![];

        if as_int {
            for (s, append, w) in &mut self.words {
                ans.push( (s.clone(), *append, w.eval_as_integer(core)?) );
            }
        }else{
            for (s, append, w) in &mut self.words {
                if as_assoc {
                    ans.push( (s.clone(), *append, w.eval_as_value(core)?) );
                }else{
                    for e in w.eval(core)? {
                        ans.push( (s.clone(), *append, e) );
                    }
                }
            }
        }
        Ok(ans)
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, sub: Option<Subscript>,
        core: &mut ShellCore, append: bool) -> bool {
        if feeder.starts_with(")") {
            return false;
        }

        let w = match Word::parse(feeder, core, None) {
            Ok(Some(w)) => w,
            _       => return false,
        };
        ans.text += &w.text;
        ans.words.push((sub, append, w));
        true
    }

    fn eat_subscript(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Self)
    -> Result<Option<Subscript>, ParseError> {
        if let Some(s) = Subscript::parse(feeder, core)? {
            if feeder.starts_with("=") {
                ans.text += &s.text.clone();
                return Ok(Some(s));
            }else if feeder.starts_with("+=") {
                ans.text += &s.text.clone();
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
        let mut paren_counter = 1;
        loop {
            let mut append = false;
            command::eat_blank_lines(feeder, core, &mut ans.text)?;

            let sub = Self::eat_subscript(feeder, core, &mut ans)?;

            if sub.is_some() {
                if feeder.starts_with("=") {
                    ans.text += &feeder.consume(1);
                }else if feeder.starts_with("+=") {
                    append = true;
                    ans.text += &feeder.consume(2);
                }
            }

            if Self::eat_word(feeder, &mut ans, sub, core, append) {
                continue;
            }

            if feeder.len() != 0 {
                if feeder.starts_with(")") {
                    paren_counter -= 1;
                    if paren_counter == 0 {
                        ans.text += &feeder.consume(1);
                        break;
                    }
                }else if feeder.starts_with("\n") {
                    ans.text += &feeder.consume(1);
                }

                let len = feeder.scanner_char();
                let err_char = feeder.consume(len);
                if &err_char == "(" {
                    paren_counter += 1;
                }
                ans.text += &err_char.clone();
                ans.error_strings.push(err_char);
                continue;

            }else if ! feeder.feed_additional_line(core).is_ok() {
                return Ok(None);
            }
        }

        Ok(Some(ans))
    }
}
