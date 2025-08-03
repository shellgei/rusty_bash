//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{BracedParam, EscapedChar, Parameter, Subword, VarName};
use crate::elements::subword::{Arithmetic, CommandSubstitution};
use crate::elements::word::{substitution, Word, WordMode};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct DoubleQuoted {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
    split_points: Vec<usize>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        self.connect_array(core)?;

        let mut word = match self.subwords.iter().any(|sw| sw.is_array()) {
            true => Word::from(self.replace_array(core)?),
            false => Word::from(self.subwords.clone()),
        };

        substitution::eval(&mut word, core)?;
        self.subwords = word.subwords;
        self.text.clear();

        for (i, sw) in self.subwords.iter_mut().enumerate() {
            if self.split_points.contains(&i) {
                self.text += " ";
            }
            self.text += sw.get_text();
        }
        Ok(())
    }

    fn make_glob_string(&mut self) -> String {
        self.text
            .replace("\\", "\\\\")
            .replace("*", "\\*")
            .replace("?", "\\?")
            .replace("[", "\\[")
            .replace("]", "\\]")
            .replace("@", "\\@")
            .replace("+", "\\+")
            .replace("!", "\\!")
    }

    fn make_unquoted_string(&mut self) -> Option<String> {
        let mut text = String::new();

        for (i, sw) in self.subwords.iter_mut().enumerate() {
            if self.split_points.contains(&i) {
                text += " ";
            }

            if let Some(txt) = sw.make_unquoted_string() {
                text += &txt;
            }
        }

        if text.is_empty() && self.split_points.len() == 1 {
            return None;
        }

        Some(text)
    }

    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)> {
        let mut ans = vec![];
        let mut last = 0;
        let mut tmp = Self::default();
        for p in &self.split_points {
            tmp.subwords = self.subwords[last..*p].to_vec();
            ans.push((tmp.boxed_clone(), true));
            last = *p;
        }
        ans
    }
}

impl DoubleQuoted {
    fn connect_array(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        for sw in self.subwords.iter_mut() {
            if sw.get_text() == "$*" || sw.get_text() == "${*}" {
                let params = core.db.get_position_params();
                let joint = core.db.get_ifs_head();
                *sw = From::from(&params.join(&joint));
            }
        }
        Ok(())
    }

    fn replace_array(&mut self, core: &mut ShellCore) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        let mut ans = vec![];

        for sw in &mut self.subwords {
            if !sw.is_array() {
                ans.push(sw.boxed_clone());
                continue;
            }

            let array = match sw.get_text() {
                "$@" | "${@}" => core.db.get_position_params(),
                _ => {
                    sw.substitute(core)?;
                    sw.get_elem()
                }
            };

            for text in array {
                ans.push(From::from(&text));
                self.split_points.push(ans.len());
            }
            self.split_points.pop();
        }

        self.split_points.push(ans.len());
        Ok(ans)
    }

    fn eat_element(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        let sw: Box<dyn Subword> = if let Some(a) = BracedParam::parse(feeder, core)? {
            Box::new(a)
        } else if let Some(a) = Arithmetic::parse(feeder, core)? {
            Box::new(a)
        } else if let Some(a) = CommandSubstitution::parse(feeder, core)? {
            Box::new(a)
        }
        //else if let Some(a) = CommandSubstitutionOld::parse(feeder, core)? {Box::new(a)}
        else if let Some(a) = Parameter::parse(feeder, core) {
            Box::new(a)
        } else if let Some(a) = Self::parse_escaped_char(feeder) {
            Box::new(a)
        } else if let Some(a) = Self::parse_name(feeder, core) {
            Box::new(a)
        } else {
            return Ok(false);
        };

        ans.text += sw.get_text();
        ans.subwords.push(sw);
        Ok(true)
    }

    fn parse_escaped_char(feeder: &mut Feeder) -> Option<EscapedChar> {
        if feeder.starts_with("\\$")
            || feeder.starts_with("\\\\")
            || feeder.starts_with("\\\"")
            || feeder.starts_with("\\`")
        {
            return Some(EscapedChar {
                text: feeder.consume(2),
            });
        }
        None
    }

    fn parse_name(feeder: &mut Feeder, core: &mut ShellCore) -> Option<VarName> {
        match feeder.scanner_name(core) {
            0 => None,
            n => Some(VarName {
                text: feeder.consume(n),
            }),
        }
    }

    fn eat_char(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        let len = feeder.scanner_char();
        if len == 0 {
            feeder.feed_additional_line(core)?;
            return Ok(true);
        }

        let ch = feeder.consume(len);
        ans.text += &ch.clone();
        if ch != "\"" {
            ans.subwords.push(From::from(&ch));
            return Ok(true);
        }
        Ok(false)
    }

    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        mode: &Option<WordMode>,
    ) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("\"") && !feeder.starts_with("$\"") {
            return Ok(None);
        }
        if let Some(WordMode::Heredoc) = mode {
            return Ok(None);
        }

        let mut ans = Self::default();
        feeder.nest.push(("\"".to_string(), vec!["\"".to_string()]));

        let len = if feeder.starts_with("\"") { 1 } else { 2 };

        ans.text = feeder.consume(len);

        while Self::eat_element(feeder, &mut ans, core)? || Self::eat_char(feeder, &mut ans, core)?
        {
        }

        feeder.nest.pop();
        Ok(Some(ans))
    }
}
