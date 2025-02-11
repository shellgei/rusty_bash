//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils::exit;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct EscapedChar {
    pub text: String,
}

impl Subword for EscapedChar {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn make_unquoted_string(&mut self) -> Option<String> {
        match self.text.len() {
            0 => exit::internal("unescaped escaped char"),
            1 => None,
            _ => Some(self.text[1..].to_string()),
        }
    }

    fn make_ansi_c_string(&mut self) -> String {
        match &self.text[1..] {
            "a" => return r"\a".to_string(),
            "b" => return r"\b".to_string(),
            "e" => return r"\e".to_string(),
            "E" => return r"\E".to_string(),
            "f" => return r"\f".to_string(),
            "n" => return "\n".to_string(),
            "r" => return "\r".to_string(),
            "t" => return "\t".to_string(),
            "v" => return r"\v".to_string(),
            "\\" => return "\\".to_string(),
            "'" => return "'".to_string(),
            "\"" => return "\"".to_string(),
            _ => {},
        }

        self.text.clone()
    }

    fn make_glob_string(&mut self) -> String {
        if let Some(c) = self.text.chars().nth(1) {
            if ! "*?[]^!\\".contains(c) {
                return c.to_string();
            }
        }
        self.text.clone()
    }
}

impl EscapedChar {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        match feeder.scanner_escaped_char(core) {
            0 => None,
            n => Some(EscapedChar{ text: feeder.consume(n) }),
        }
    }
}
