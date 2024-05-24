//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct EscapedChar {
    pub text: String,
}

impl Subword for EscapedChar {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn merge(&mut self, right: &Box<dyn Subword>) {
        self.text += &right.get_text();
    }

    /*
    fn set(&mut self, subword_type: SubwordType, s: &str){
        self.text = s.to_string();
    }*/

    /*
    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        match self.subword_type {
            SubwordType::Parameter => {
                let value = core.data.get_param(&self.text[1..]);
                self.text = value.to_string();
            },
            _ => {},
        }
        true
    }*/

    fn make_glob_string(&mut self) -> String { self.text.clone() }

    fn make_unquoted_string(&mut self) -> String {
        self.text[1..].to_string()
    }

    fn get_type(&self) -> SubwordType { SubwordType::EscapedChar  }
    fn clear(&mut self) { self.text = String::new(); }
}

impl EscapedChar {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let len = feeder.scanner_escaped_char(core);
        match len > 0 {
            true  => Some(EscapedChar{ text: feeder.consume(len) }),
            false => None,
        }
    }
}
