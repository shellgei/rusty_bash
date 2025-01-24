//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::Subword;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    command: ParenCommand,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, _: &mut ShellCore) -> Result<(), ExecError> {
        dbg!("{:?}", &self.text);
        Ok(())
    }
}

impl CommandSubstitution {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
                         -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("$(") {
            return Ok(None);
        }
        let mut text = feeder.consume(1);

        if let Some(pc) = ParenCommand::parse(feeder, core)? {
            text += &pc.get_text();
            Ok(Some(Self {text: text, command: pc} ))
        }else{
            Ok(None)
        }
    }
}
