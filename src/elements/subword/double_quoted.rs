//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{SimpleSubword, Subword};

#[derive(Debug, Clone, Default)]
pub struct DoubleQuoted {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

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

        while Self::eat_char(feeder, &mut ans, core)? {}

        Ok(Some(ans))
    }
}
