//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct Parameter {
    pub text: String,
}

impl Subword for Parameter {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        let value = core.data.get_param(&self.text[1..]);
        self.text = value.to_string();
        true
    }

    fn get_type(&self) -> SubwordType { SubwordType::Parameter  }
}

impl Parameter {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let len = feeder.scanner_dollar_special_and_positional_param(core);
        match len > 0 {
            true  => Some(Self { text: feeder.consume(len) } ),
            false => None,
        }
    }
}
