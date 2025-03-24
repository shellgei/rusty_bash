//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use crate::elements::word::{Word, substitution};
use crate::elements::subword::CommandSubstitution;
use crate::elements::subword::Arithmetic;
use super::{BracedParam, EscapedChar, SimpleSubword, Parameter, Subword, VarName};

#[derive(Debug, Clone, Default)]
pub struct DoubleQuoted {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
    split_points: Vec<usize>,
    array_empty: bool,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore)
    -> Result<Vec<Box<dyn Subword>>, ExecError> {
        let mut word = match self.subwords.iter().any(|sw| sw.is_array()) {
            true  => Word::from(self.replace_array(core)?),
            false => Word::from(self.subwords.clone()),
        };

        substitution::eval(&mut word, core)?;
        self.subwords = word.subwords;
        self.text = word.text;
        Ok(vec![])
    }

    fn make_glob_string(&mut self) -> String {
        return self.text.replace("\\", "\\\\")
                        .replace("*", "\\*")
                        .replace("?", "\\?")
                        .replace("[", "\\[")
                        .replace("]", "\\]");
    }

    fn make_unquoted_string(&mut self) -> Option<String> {
        let text = self.subwords.iter_mut()
            .map(|s| s.make_unquoted_string())
            .filter(|s| *s != None)
            .map(|s| s.unwrap())
            .collect::<Vec<String>>()
            .concat();

        if text.is_empty() && self.array_empty {
            return None;
        }
        Some(text)
    }

    fn split(&self, _: &str) -> Vec<Box<dyn Subword>>{
        let mut ans = vec![];
        let mut last = 0;
        let mut tmp = Self::default();
        for p in &self.split_points {
            tmp.subwords = self.subwords[last..*p].to_vec();
            ans.push(tmp.boxed_clone());
            last = *p;
        }
        ans
    }
}

impl DoubleQuoted {
    fn replace_array(&mut self, core: &mut ShellCore) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        let mut ans = vec![];
        self.array_empty = true;

        for sw in &mut self.subwords {
            if ! sw.is_array() {
                ans.push(sw.boxed_clone());
                continue;
            }

            let array = match sw.get_text() {
                "$@" | "${@}" => core.db.get_position_params(),
                _ => {
                    sw.substitute(core)?;
                    sw.get_array_elem()
                },
            };

            for text in array {
                self.array_empty = false;
                ans.push(SimpleSubword{text}.boxed_clone());
                self.split_points.push(ans.len());
            }
            self.split_points.pop();
        }

        self.split_points.push(ans.len());
        Ok(ans)
    }

    fn eat_element(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        let sw: Box<dyn Subword> 
            = if let Some(a) = BracedParam::parse(feeder, core)? {Box::new(a)}
            else if let Some(a) = Arithmetic::parse(feeder, core)? {Box::new(a)}
            else if let Some(a) = CommandSubstitution::parse(feeder, core)? {Box::new(a)}
            else if let Some(a) = Parameter::parse(feeder, core) {Box::new(a)}
            else if let Some(a) = Self::parse_escaped_char(feeder, core) { a }
            else if let Some(a) = Self::parse_name(feeder, core) { Box::new(a) }
            else { return Ok(false) ; };

        ans.text += sw.get_text();
        ans.subwords.push(sw);
        Ok(true)
    }

    fn parse_escaped_char(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
        if feeder.starts_with("\\$") || feeder.starts_with("\\\\") 
        || feeder.starts_with("\\\"") || feeder.starts_with("\\`") {
            return Some(Box::new(EscapedChar{ text: feeder.consume(2) }));
        }
        match feeder.scanner_escaped_char(core) {
            0 => None,
            n => Some(Box::new(SimpleSubword{text: feeder.consume(n)})),
        }
    }

    fn parse_name(feeder: &mut Feeder, core: &mut ShellCore) -> Option<VarName> {
        match feeder.scanner_name(core) {
            0 => None,
            n => Some(VarName{ text: feeder.consume(n) }),
        }
    }

    fn eat_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        match feeder.nth(0) {
            Some('"') => {
                ans.text += &feeder.consume(1);
                return Ok(false);
            },
            Some(ch) => {
                let txt = feeder.consume(ch.len_utf8());
                ans.text += &txt;
                ans.subwords.push( Box::new(SimpleSubword{ text: txt }) );
                return Ok(true);
            },
            None     => feeder.feed_additional_line(core)?,
        }
        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("\"") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text = feeder.consume(1);

        while Self::eat_element(feeder, &mut ans, core)?
           || Self::eat_char(feeder, &mut ans, core)? {}

        Ok(Some(ans))
    }
}
