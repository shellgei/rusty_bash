//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{EscapedChar, SimpleSubword, Parameter, Subword, VarName};

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
}

impl DoubleQuoted {
    fn set_simple_subword(feeder: &mut Feeder, ans: &mut Self, len: usize) -> bool {
        if len == 0 {
            return false;
        }

        let txt = feeder.consume(len);
        ans.text += &txt;
        ans.subwords.push( Box::new(SimpleSubword{ text: txt }) );
        true
    }

    fn eat_escaped_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.starts_with("\\$") 
        || feeder.starts_with("\\\\") 
        || feeder.starts_with("\\\"") 
        || feeder.starts_with("\\`") {
            let txt = feeder.consume(2);
            ans.text += &txt;
            ans.subwords.push(Box::new(EscapedChar{ text: txt }));
            return true;
        }
        let len = feeder.scanner_escaped_char(core);
        Self::set_simple_subword(feeder, ans, len)
    }

    fn eat_other(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_double_quoted_subword(core);
        Self::set_simple_subword(feeder, ans, len)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("\"") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text = feeder.consume(1);

        loop {
            while Self::eat_escaped_char(feeder, &mut ans, core)
               || Self::eat_other(feeder, &mut ans, core) {}

            if feeder.starts_with("\"") {
                ans.text += &feeder.consume(1);
                return Ok(Some(ans));
            }else if feeder.len() > 0 {
                return Err(ParseError::UnexpectedSymbol(feeder.consume(feeder.len())));
            }else{
                feeder.feed_additional_line(core)?;
            }
        }
    }
}
