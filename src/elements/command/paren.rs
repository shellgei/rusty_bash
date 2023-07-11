//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::Command;
use super::Pipe;

#[derive(Debug)]
pub struct ParenCommand {
    pub text: String,
    pub script: Option<Script>,
}

impl Command for ParenCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) {
        match self.script {
            Some(ref mut s) => s.fork_exec(core, pipe),
            _               => panic!("SUSH INTERNAL ERROR (ParenCommand::exec)"),
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl ParenCommand {
    fn new() -> ParenCommand {
        ParenCommand {
            text: String::new(),
            script: None,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ParenCommand> {
        match Script::parse_nested(feeder, core, "(") {
            Some(s) => {
                let mut ans = Self::new();
                ans.text = "(".to_string() + &s.text.clone() + &feeder.consume(1);
                ans.script = Some(s);
                Some(ans)
            },
            None => None, 
        }
    }
}
