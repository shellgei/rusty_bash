//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use crate::elements::word::{Word, WordMode};
use crate::elements::subword::{Arithmetic, CommandSubstitution};
use super::{BracedParam, EscapedChar, Parameter, Subword, VarName};

#[derive(Debug, Clone, Default)]
pub struct EvalLetParen {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for EvalLetParen {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        self.connect_array(core)?;

        let word = Word::from(self.subwords.clone());
        self.text = word.eval_as_value(core)?;
        Ok(())
    }

    /*
    fn make_glob_string(&mut self) -> String {
        return self.text.replace("\\", "\\\\")
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

    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{
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
    */
}

impl EvalLetParen {
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

    fn eat_element(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        let sw: Box<dyn Subword> 
            = if let Some(a) = BracedParam::parse(feeder, core)? {Box::new(a)}
            else if let Some(a) = Arithmetic::parse(feeder, core)? {Box::new(a)}
            else if let Some(a) = CommandSubstitution::parse(feeder, core)? {Box::new(a)}
            else if let Some(a) = Parameter::parse(feeder, core) {Box::new(a)}
            else if let Some(a) = EscapedChar::parse(feeder, core){ Box::new(a) }
            else if let Some(a) = Self::parse_name(feeder, core) { Box::new(a) }
            else { return Ok(false) ; };

        ans.text += sw.get_text();
        ans.subwords.push(sw);
        Ok(true)
    }
/*
    fn parse_escaped_char(feeder: &mut Feeder) -> Option<EscapedChar> {
        if feeder.starts_with("\\$") || feeder.starts_with("\\\\") 
        || feeder.starts_with("\\\"") || feeder.starts_with("\\`") {
            return Some(EscapedChar{ text: feeder.consume(2) });
        }
        None
    }*/

    fn parse_name(feeder: &mut Feeder, core: &mut ShellCore) -> Option<VarName> {
        match feeder.scanner_name(core) {
            0 => None,
            n => Some(VarName{ text: feeder.consume(n) }),
        }
    }

    fn eat_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        if feeder.starts_with(")") {
            ans.text += &feeder.consume(1);
            ans.subwords.push( From::from(")") );
            return Ok(false);
        }

        let len = feeder.scanner_char();
        if len == 0 {
            feeder.feed_additional_line(core)?;
            return Ok(true);
        }

        let ch = feeder.consume(len);
        ans.text += &ch.clone();
        ans.subwords.push( From::from(&ch) );
        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore,
                 mode: &Option<WordMode>) -> Result<Option<Self>, ParseError> {
        match mode {
            Some(WordMode::EvalLet) => {},
            _ => return Ok(None),
        }

        if ! feeder.starts_with("(") {
            return Ok(None);
        }

        let mut ans = Self::default();

        while Self::eat_element(feeder, &mut ans, core)?
           || Self::eat_char(feeder, &mut ans, core)? {}

        Ok(Some(ans))
    }
}
