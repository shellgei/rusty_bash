//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils::error::ParseError;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct VarName {
    pub text: String,
}

impl Subword for VarName {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn is_name(&self) -> bool {true}
}

impl VarName {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        match feeder.scanner_name(core) {
            0 => Ok(None),
            n => Ok(Some( Self{ text: feeder.consume(n) } )),
        }
    }
}
