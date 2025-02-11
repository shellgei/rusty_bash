//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{Subword, SimpleSubword, EscapedChar};

#[derive(Debug, Clone, Default)]
pub struct AnsiCQuoted {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Subword for AnsiCQuoted {
    fn get_text(&self) -> &str {&self.text}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn make_unquoted_string(&mut self) -> Option<String> {
        let mut ans = String::new();
        for sw in &mut self.subwords {
            ans += &sw.make_ansi_c_string();
        }
        Some(ans)
    }

    fn make_glob_string(&mut self) -> String {
        self.text[2..self.text.len()-1].replace("\\", "\\\\")
            .replace("*", "\\*")
            .replace("?", "\\?")
            .replace("[", "\\[")
            .replace("]", "\\]")
    }

    fn split(&self) -> Vec<Box<dyn Subword>>{ vec![] }
}

impl AnsiCQuoted {
    fn eat_simple_subword(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if let Some(a) = SimpleSubword::parse(feeder) {
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            true
        }else{
            false
        }
    }

    fn eat_escaped_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(a) = EscapedChar::parse(feeder, core) {
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
                          -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("$'") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        while ! feeder.starts_with("'") {
            if Self::eat_simple_subword(feeder, &mut ans) 
            || Self::eat_escaped_char(feeder, &mut ans, core) {
                continue;
            }

            if feeder.len() == 0 {
                feeder.feed_additional_line(core)?;
                continue;
            }
        
            let other = feeder.consume(1);
            ans.text += &other.clone();
            ans.subwords.push( Box::new(SimpleSubword{ text: other }));
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
