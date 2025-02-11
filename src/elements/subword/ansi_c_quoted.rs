//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::Subword;

#[derive(Debug, Clone, Default)]
pub struct AnsiCQuoted {
    pub text: String,
}

impl Subword for AnsiCQuoted {
    fn get_text(&self) -> &str {&self.text}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn make_unquoted_string(&mut self) -> Option<String> {
        Some( self.text[2..self.text.len()-1].to_string() )
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
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
                          -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("$'") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        while ! feeder.starts_with("'") {
            let len = feeder.scanner_subword();
            if len > 0 {
                ans.text += &feeder.consume(len);
                continue;
            }
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
