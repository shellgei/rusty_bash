//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::word::{substitution, Word};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::{CommandSubstitution, Parameter, SimpleSubword, Subword};

#[derive(Debug, Clone, Default)]
pub struct DoubleQuoted {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut word = Word::from(self.subwords.clone());
        substitution::eval(&mut word, core)?;
        self.subwords = word.subwords;
        dbg!("{:?}", &self.subwords);
        self.text = word.text;
        Ok(())
    }

    fn make_unquoted_string(&mut self) -> Option<String> {
        Some( self.text[1..self.text.len()-1].to_string() )
    }

    fn make_glob_string(&mut self) -> String {
        self.text[1..self.text.len()-1].replace("\\", "\\\\")
            .replace("*", "\\*")
            .replace("?", "\\?")
            .replace("[", "\\[")
            .replace("]", "\\]")
    }

    fn split(&self) -> Vec<Box<dyn Subword>>{ vec![] }
}

impl DoubleQuoted {
    fn eat_element(feeder: &mut Feeder, ans: &mut Self,
                   core: &mut ShellCore) -> Result<bool, ParseError> {
        let sw: Box<dyn Subword>
            = if let Some(a) = CommandSubstitution::parse(feeder, core)? {Box::new(a)}
            else if let Some(a) = Parameter::parse(feeder, core) {Box::new(a)}
            else { return Ok(false) ; };

        ans.text += sw.get_text();
        ans.subwords.push(sw);
        Ok(true)
    }

    fn eat_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        let len = feeder.scanner_char();
        if len == 0 {
            feeder.feed_additional_line(core)?;
            return Ok(true);
        }

        let ch = feeder.consume(len);
        ans.text += &ch.clone();
        if ch != "\"" {
            ans.subwords.push( Box::new(SimpleSubword{ text: ch }) );
            return Ok(true);
        }
        Ok(false)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("\"") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text = feeder.consume(1);

        while Self::eat_element(feeder, &mut ans, core)?
           || Self::eat_char(feeder, &mut ans, core)? {}

        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
