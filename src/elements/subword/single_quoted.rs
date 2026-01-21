//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Subword;
use crate::{Feeder, ShellCore};
use crate::elements::word::WordMode;

#[derive(Debug, Clone)]
pub struct SingleQuoted {
    pub text: String,
}

impl Subword for SingleQuoted {
    fn get_text(&self) -> &str {
        &self.text
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn make_unquoted_string(&mut self) -> Option<String> {
        Some(self.text[1..self.text.len() - 1].to_string())
    }

    fn make_glob_string(&mut self) -> String {
        self.text[1..self.text.len() - 1]
            .replace("\\", "\\\\")
            .replace("*", "\\*")
            .replace("?", "\\?")
            .replace("[", "\\[")
            .replace("]", "\\]")
    }

    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)> {
        vec![]
    }
}

impl SingleQuoted {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore,
                 mode: &Option<WordMode>) -> Option<Self> {
        if let Some(WordMode::AlterWord) = mode {
            if core.options.query("posix") {
                return None;
            }
        }
        
        match feeder.scanner_single_quoted_subword(core) {
            0 => None,
            n => {
                let s = feeder.consume(n);
                Some(SingleQuoted { text: s })
            }
        }
    }
}
