//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{Subword, SimpleSubword, EscapedChar};
use crate::codec::ansi_c::AnsiCToken;

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
    fn eat_simple_subword(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if let Some(a) = SimpleSubword::parse(feeder) {
            ans.text += a.get_text();
            ans.tokens.push(AnsiCToken::Normal(a.get_text().to_string()));
            true
        }else{
            false
        }
    }

    fn eat_oct(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut len = feeder.scanner_ansi_c_oct(core);
        if len < 2 {
            return false;
        }

        if len > 4 {
            len = 4;
        }

        let token = feeder.consume(len);
        ans.text += &token.clone();
        ans.tokens.push( AnsiCToken::Oct(token[1..].to_string()));
        true
    }

    fn eat_hex_braced(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with("\\x{") {
            return false;
        }

        if feeder.starts_with("\\x{}") {
            ans.text += &feeder.consume(4);
            ans.tokens.push(AnsiCToken::EmptyHex);
            return true;
        }

        let len = feeder.scanner_ansi_c_hex(core);
        if len < 4 {
            return false;
        }

        let mut token = feeder.consume(len);
        ans.text += &token.clone();
        token.retain(|c| c != '}' && c != '{');
        if token.len() > 3 {
            ans.tokens.push( AnsiCToken::Hex(token[3..].to_string()));
        }
        true

    }

    fn eat_hex(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut len = feeder.scanner_ansi_c_hex(core);
        if len < 3 {
            return false;
        }

        if len > 4 {
            len = 4;
        }

        let token = feeder.consume(len);
        ans.text += &token.clone();
        ans.tokens.push( AnsiCToken::Hex(token[2..].to_string()));
        true
    }

    fn eat_unicode4(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut len = feeder.scanner_ansi_unicode4(core);
        if len < 3 {
            return false;
        }

        if len > 6 {
            len = 6;
        }

        let token = feeder.consume(len);
        ans.text += &token.clone();
        ans.tokens.push( AnsiCToken::Unicode4(token[2..].to_string()));
        true
    }

    fn eat_unicode8(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut len = feeder.scanner_ansi_unicode8(core);
        if len < 3 {
            return false;
        }

        if len > 10 {
            len = 10;
        }

        let token = feeder.consume(len);
        ans.text += &token.clone();
        ans.tokens.push( AnsiCToken::Unicode8(token[2..].to_string()));
        true
    }

    fn eat_escaped_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(a) = EscapedChar::parse(feeder, core) {
            let txt = a.get_text().to_string();
            ans.text += &txt.clone();

            if txt != "\\c" || feeder.len() == 0 {
                ans.tokens.push(AnsiCToken::OtherEscaped(txt[1..].to_string()));
            }else{
                if let Some(a) = EscapedChar::parse(feeder, core) {
                    let mut text_after = a.get_text().to_string();
                    ans.text += &text_after.clone();
                    text_after.remove(0);
                    let ctrl_c = text_after.chars().nth(0).unwrap();
                    ans.tokens.push(AnsiCToken::Control(ctrl_c));
                }else if feeder.starts_with("'") {
                    ans.tokens.push(AnsiCToken::Normal("\\c".to_string()));
                }else{
                    let ctrl_c = feeder.consume(1).chars().nth(0).unwrap();
                    ans.text += &ctrl_c.to_string();
                    ans.tokens.push(AnsiCToken::Control(ctrl_c));
                }
            }
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("$'") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        while ! feeder.starts_with("'") {
            if Self::eat_simple_subword(feeder, &mut ans) 
            || Self::eat_hex_braced(feeder, &mut ans, core)
            || Self::eat_hex(feeder, &mut ans, core)
            || Self::eat_oct(feeder, &mut ans, core)
            || Self::eat_unicode4(feeder, &mut ans, core)
            || Self::eat_unicode8(feeder, &mut ans, core)
            || Self::eat_escaped_char(feeder, &mut ans, core) {
                continue;
            }

            if feeder.len() == 0 {
                feeder.feed_additional_line(core)?;
                continue;
            }
        
            let other = feeder.consume(1);
            ans.text += &other.clone();
            ans.tokens.push(AnsiCToken::Normal(other));
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
