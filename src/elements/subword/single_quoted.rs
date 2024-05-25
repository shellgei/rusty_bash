//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct SingleQuoted {
    text: String,
}

impl Subword for SingleQuoted {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn make_glob_string(&mut self) -> String {
        let ans = self.text
            .replace("\\", "\\\\")
            .replace("*", "\\*")
            .replace("?", "\\?")
            .replace("[", "\\[")
            .replace("]", "\\]");
        let len = ans.len();
        ans[1..len-1].to_string()
    }

    fn make_unquoted_string(&mut self) -> String {
        let len = self.text.len();
        self.text[1..len-1].to_string()
    }

    fn get_type(&self) -> SubwordType { SubwordType::SingleQuoted  }
}

impl SingleQuoted {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let len = feeder.scanner_single_quoted_subword(core);
        match len > 0 {
            true  => Some(SingleQuoted{ text: feeder.consume(len) }),
            false => None,
        }
    }
}
