//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::elements::subword::escaped_char::EscapedChar;
use crate::elements::subword::simple::SimpleSubword;
use crate::elements::subword::Subword;

#[derive(Clone, Debug)]
pub enum AnsiCToken {
    Normal(String),
    Oct(String),
    Hex(String),
    EmptyHex,
    Unicode4(String),
    Unicode8(String),
    Control(char),
    OtherEscaped(String),
}

impl AnsiCToken {
    pub fn to_string(&mut self) -> String {
        match &self {
            AnsiCToken::EmptyHex => String::new(),
            AnsiCToken::Normal(s) => s.clone(), 
            AnsiCToken::Oct(s) => {
                let mut num = u32::from_str_radix(&s, 8).unwrap();
                if num >= 256 {
                    num -= 256;
                }

                if num >= 128 {
                    num += 0xE000;
                    char::from_u32(num).unwrap().to_string()
                }else{
                    char::from(num as u8).to_string()
                }
            },
            AnsiCToken::Hex(s) => {
                let hex = match s.len() > 2 {
                    true  => s[s.len()-2..].to_string(),
                    false => s.to_string(),
                };

                let mut num = match u32::from_str_radix(&hex, 16) {
                    Ok(n) => n,
                    _ => return String::new(),
                };
                if num >= 256 {
                    num -= 256;
                }

                if num >= 128 {
                    num += 0xE000;
                    char::from_u32(num).unwrap().to_string()
                }else{
                    char::from(num as u8).to_string()
                }
            },
            AnsiCToken::Unicode4(s) => {
                let num = u32::from_str_radix(&s, 16).unwrap();
                match char::from_u32(num) {
                    Some(c) => c.to_string(),
                    _ => "U+".to_owned() + s,
                }
            },
            AnsiCToken::Unicode8(s) => {
                let num = u64::from_str_radix(&s, 16).unwrap();
                let mut ans = String::new();
                for i in (0..4).rev() {
                    let n = (((num >> i*8) & 0xFF) + 0xE000 + ((i+1) << 8)) as u32;
                    ans.push( char::from_u32(n).unwrap() );
                }
                //unsafe { char::from_u32_unchecked(num as u32) }.to_string()
                ans
            },
            AnsiCToken::Control(c) => {
                let num = if *c == '@' { 0 }
                    else if *c == '[' { 27 }
                    else if *c == '\\' { 28 }
                    else if *c == ']' { 29 }
                    else if *c == '^' { 30 }
                    else if *c == '_' { 31 }
                    else if *c == '?' { 127 }
                    else if '0' <= *c && *c <= '9' { *c as u32 - 32 }
                    else if 'a' <= *c && *c <= 'z' { *c as u32 - 96 }
                    else if 'A' <= *c && *c <= 'Z' { *c as u32 - 64 }
                    else if *c as u32 >= 32 { *c as u32 - 32 }
                    else{ *c as u32 };

                char::from_u32(num).unwrap().to_string()
            },
            AnsiCToken::OtherEscaped(s) => match s.as_ref() {
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
pub struct AnsiCString {
    pub text: String,
    pub tokens: Vec<AnsiCToken>,
}

impl AnsiCString {
    pub fn eval(&mut self) -> String {
        let mut ans = String::new();
        for t in &mut self.tokens {
            if let AnsiCToken::EmptyHex = t {
                break;
            }
            ans += &t.to_string();
        }
        ans
    }

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

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore, end: Option<String>)
    -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        loop {
            if let Some(ref e) = end {
                if feeder.starts_with(&e) {
                    break;
                }
            }

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
                if end.is_none() {
                    break;
                }
                feeder.feed_additional_line(core)?;
                continue;
            }
        
            let other = feeder.consume(1);
            ans.text += &other.clone();
            ans.tokens.push(AnsiCToken::Normal(other));
        }

        Ok(Some(ans))
    }
}
