//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct VarName {
    pub text: String,
}

impl Subword for VarName {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn get_type(&self) -> SubwordType { SubwordType::VarName  }
}

impl VarName {
    pub fn new(s: &str) -> VarName {
        VarName {
            text: s.to_string(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<VarName> {
        let len = feeder.scanner_name(core);
        match len > 0 {
            true  => Some(Self::new(&feeder.consume(len))),
            false => None,
        }
    }
}
