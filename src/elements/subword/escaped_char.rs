//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use crate::utils::exit;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone)]
pub struct EscapedChar {
    pub text: String,
}

impl Subword for EscapedChar {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn make_unquoted_string(&mut self) -> Option<String> {
        match self.text.len() {
            0 => exit::internal("unescaped escaped char"),
            1 => None,
            _ => Some(self.text[1..].to_string()),
        }
    }

    fn make_glob_string(&mut self) -> String {
        if let Some(c) = self.text.chars().nth(1) {
            if !"*?[]^!\\".contains(c) {
                return c.to_string();
            }
        }
        self.text.clone()
    }

    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)> {
        vec![]
    }
}

impl EscapedChar {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        match feeder.scanner_escaped_char(core) {
            0 => None,
            n => Some(EscapedChar {
                text: feeder.consume(n),
            }),
        }
    }
}
