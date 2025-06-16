//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

/*
use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{Subword, SimpleSubword, EscapedChar};
*/

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
