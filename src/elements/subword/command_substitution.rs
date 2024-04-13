//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Script, ShellCore, Feeder};
use crate::elements::command;
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    script: Option<Script>,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitution(&mut self, core: &mut ShellCore) -> bool {
        match self.script.as_mut() {
            Some(s) => {
                s.exec(core);
                true
            },
            _ => false
        }
    }

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
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "$(", vec![")"], &mut ans.script) {
            ans.text.push_str("$(");
            ans.text.push_str(&ans.script.as_ref().unwrap().get_text());
            ans.text.push_str(&feeder.consume(1));
            Some(ans)
        }else{
            None
        }
    }
}
