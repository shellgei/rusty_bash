//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{Subword, SimpleSubword, EscapedChar};

#[derive(Clone, Debug)]
enum Token {
    Normal(String),
    Oct(String),
    Hex(String),
    Unicode4(String),
    Unicode8(String),
    Control(char),
    OtherEscaped(String),
}

impl Token {
    fn to_string(&mut self) -> String {
        match &self {
            Token::Normal(s) => s.clone(), 
            Token::Oct(s) => {
                let mut num = u32::from_str_radix(&s, 8).unwrap();
                if num >= 256 {
                    num -= 256;
                }
                char::from(num as u8).to_string() //MEMO (differece from Bash)
                                                  //128-255 are never straightly converted 
                                                  //because a binary 1.... is a reserved number in UTF-8
            },
            Token::Hex(s) => {
                let mut num = u32::from_str_radix(&s, 16).unwrap();
                if num >= 256 {
                    num -= 256;
                }
                char::from(num as u8).to_string()
            },
            Token::Unicode4(s) => {
                let num = u32::from_str_radix(&s, 16).unwrap();
                match char::from_u32(num) {
                    Some(c) => c.to_string(),
                    _ => "U+".to_owned() + s,
                }
            },
            Token::Unicode8(s) => {
                let num = u64::from_str_radix(&s, 16).unwrap();
                unsafe { char::from_u32_unchecked(num as u32) }.to_string()
            },
            Token::Control(c) => {
                let num = if *c == '@' { 0 }
                    else if *c == '[' { 27 }
                    else if *c == '\\' { 28 }
                    else if *c == ']' { 29 }
                    else if *c == '^' { 30 }
                    else if *c == '?' { 127 }
                    else if '0' <= *c && *c <= '9' { *c as u32 - 32 }
                    else if 'a' <= *c && *c <= 'z' { *c as u32 - 96 }
                    else if 'A' <= *c && *c <= 'Z' { *c as u32 - 64 }
                    else if *c as u32 >= 32 { *c as u32 - 32 }
                    else{ *c as u32 };

                char::from_u32(num).unwrap().to_string()
            },
            Token::OtherEscaped(s) => match s.as_ref() {
                "a" => char::from(7).to_string(),
                "b" =>  char::from(8).to_string(),
                "e" | "E" => char::from(27).to_string(),
                "f" => char::from(12).to_string(),
                "n" => "\n".to_string(),
                "r" => "\r".to_string(),
                "t" => "\t".to_string(),
                "v" => char::from(11).to_string(),
                "\\" => "\\".to_string(),
                "'" => "'".to_string(),
                "\"" => "\"".to_string(),
                _ => ("\\".to_owned() + s).to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnsiCQuoted {
    text: String,
    tokens: Vec<Token>,
}

impl Subword for AnsiCQuoted {
    fn get_text(&self) -> &str {&self.text}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn make_unquoted_string(&mut self) -> Option<String> { Some( self.make_glob_string() ) }

    fn make_glob_string(&mut self) -> String {
        let mut ans = String::new();
        for t in &mut self.tokens {
            ans += &t.to_string();
        }

        ans
    }

    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ vec![] }
}

impl AnsiCQuoted {
    fn eat_simple_subword(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if let Some(a) = SimpleSubword::parse(feeder) {
            ans.text += a.get_text();
            ans.tokens.push(Token::Normal(a.get_text().to_string()));
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
        ans.tokens.push( Token::Oct(token[1..].to_string()));
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
        ans.tokens.push( Token::Hex(token[2..].to_string()));
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
        ans.tokens.push( Token::Unicode4(token[2..].to_string()));
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
        ans.tokens.push( Token::Unicode8(token[2..].to_string()));
        true
    }

    fn eat_escaped_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(a) = EscapedChar::parse(feeder, core) {
            let txt = a.get_text().to_string();
            ans.text += &txt.clone();

            if txt != "\\c" || feeder.len() == 0 {
                ans.tokens.push(Token::OtherEscaped(txt[1..].to_string()));
            }else{
                if let Some(a) = EscapedChar::parse(feeder, core) {
                    let mut text_after = a.get_text().to_string();
                    ans.text += &text_after.clone();
                    text_after.remove(0);
                    let ctrl_c = text_after.chars().nth(0).unwrap();
                    ans.tokens.push(Token::Control(ctrl_c));
                }else{
                    let ctrl_c = feeder.consume(1).chars().nth(0).unwrap();
                    ans.text += &ctrl_c.to_string();
                    ans.tokens.push(Token::Control(ctrl_c));
                }
            }
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
            ans.tokens.push(Token::Normal(other));
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
