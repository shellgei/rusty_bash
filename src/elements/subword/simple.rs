//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct SimpleSubword {
    pub text: String,
    pub subword_type: SubwordType,
}

impl Subword for SimpleSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    //fn merge(&mut self, right: &Box<dyn Subword>) { self.text += &right.get_text(); }
    fn get_type(&self) -> SubwordType { self.subword_type.clone()  }
    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }
}

impl SimpleSubword {
    pub fn new(s: &str, tp: SubwordType) -> SimpleSubword {
        SimpleSubword {
            text: s.to_string(),
            subword_type: tp,
        }
    }

    pub fn replace_expansion(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_history_expansion(core);
        if len == 0 {
            return false;
        }

        let history_len = core.history.len();
        if history_len < 2 {
            feeder.replace(len, "");
            return true;
        }

        let mut his = String::new();
        for h in &core.history[1..] {
            let last = h.split(" ").last().unwrap();

            if ! last.starts_with("!$") {
                his = last.to_string();
                break;
            }
        }

        feeder.replace(len, &his);
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleSubword> {
        if Self::replace_expansion(feeder, core) {
            return Self::parse(feeder, core);
        }

        let len = feeder.scanner_name(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::VarName));
        }

        let len = feeder.scanner_subword_symbol();
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Simple));
        }

        let len = feeder.scanner_subword();
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Simple));
        }

        None
    }
}
