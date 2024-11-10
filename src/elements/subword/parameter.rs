//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::Subword;

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

    fn is_array(&self) -> bool {self.text == "$@"}
}

impl Parameter {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        match feeder.scanner_dollar_special_and_positional_param(core) {
            0 => None,
            n => Some(Self { text: feeder.consume(n) } ),
        }
    }
}
