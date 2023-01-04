//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elem_command::Command;
use crate::{Feeder, ShellCore};

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub text: String,
}

impl Pipeline {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for command in self.commands.iter_mut() {
            command.exec(core);
        }
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        if let Some(command) = Command::parse(text, core){
            return Some( Pipeline{text: command.text.clone(), commands: vec!(command)} );
        }
        None
    }
}
