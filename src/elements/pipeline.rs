//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use super::simple_command::SimpleCommand;

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<SimpleCommand>,
    pub text: String,
}

impl Pipeline {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for command in self.commands.iter_mut() {
            command.exec(core);
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        if let Some(command) = SimpleCommand::parse(feeder, core){
            return Some( Pipeline{text: command.text.clone(), commands: vec!(command)} );
        }
        None
    }
}
