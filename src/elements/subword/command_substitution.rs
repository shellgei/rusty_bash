//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    command: Option<ParenCommand>,
    pipe: Pipe,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitution(&mut self, core: &mut ShellCore) -> bool {
        match self.command {
            Some(_) => {
                self.text = self.exec(core);
                true
            },
            _ => false,
        }
    }

    fn get_type(&self) -> SubwordType { SubwordType::CommandSubstitution }
    fn clear(&mut self) { self.text = String::new(); }
}

impl CommandSubstitution {
    fn new() -> CommandSubstitution {
        CommandSubstitution {
            text: String::new(),
            command: None,
            pipe: Pipe::new("|".to_string()),
        }
    }

    fn exec(&mut self, core: &mut ShellCore) -> String {
        match self.command.as_mut() {
            Some(c) => {c.exec(core, &mut self.pipe);},
            _ => {},
        }
        "ok".to_string()
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("$(") {
            return None;
        }
        let mut ans = CommandSubstitution::new();
        ans.text = feeder.consume(1);

        if let Some(pc) = ParenCommand::parse(feeder, core) {
            ans.command = Some(pc);
            Some(ans)
        }else{
            None
        }
    }
}
