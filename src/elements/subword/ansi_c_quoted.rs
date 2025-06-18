//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::Subword;
use crate::elements::ansi_c_str::{AnsiCString, AnsiCToken};

#[derive(Debug, Clone, Default)]
pub struct AnsiCQuoted {
    text: String,
    tokens: Vec<AnsiCToken>,
    in_heredoc: bool, 
}

impl Subword for AnsiCQuoted {
    fn get_text(&self) -> &str {&self.text}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn make_unquoted_string(&mut self) -> Option<String> { Some(self.make_glob_string()) }

    fn make_glob_string(&mut self) -> String {
        if self.in_heredoc {
            return self.text.clone();
        }

        let mut ans = String::new();
        for t in &mut self.tokens {
            if let AnsiCToken::EmptyHex = t {
                break;
            }
            ans += &t.to_string();
        }

        ans
    }

    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ vec![] }

    fn set_heredoc_flag(&mut self) {self.in_heredoc = true; }
}

impl AnsiCQuoted {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("$'") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        if let Some(ansi_c_str) 
            = AnsiCString::parse(feeder, core, Some("'".to_string()))? {
            ans.text += &ansi_c_str.text;
            ans.tokens = ansi_c_str.tokens;
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
