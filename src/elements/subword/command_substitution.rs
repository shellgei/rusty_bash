//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Script, ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    script: Option<Script>,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn substitution(&mut self, core: &mut ShellCore) -> bool {true}
    fn get_type(&self) -> SubwordType { SubwordType::CommandSubstitution }
    fn clear(&mut self) { self.text = String::new(); }
}

impl CommandSubstitution {
    fn new() -> CommandSubstitution {
        CommandSubstitution {
            text: String::new(),
            script: None,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        None
    }
}
