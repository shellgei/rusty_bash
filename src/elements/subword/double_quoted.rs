//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct DoubleQuoted {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn parameter_expansion(&mut self, core: &mut ShellCore) -> bool {false}
    fn get_type(&self) -> SubwordType { SubwordType::DoubleQuoted  }
}

impl DoubleQuoted {
    pub fn new() -> DoubleQuoted {
        DoubleQuoted {
            text: String::new(),
            subwords: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        None
    }
}
