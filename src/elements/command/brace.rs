//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::Command;
use crate::elements::command;
use super::Pipe;

#[derive(Debug)]
pub struct BraceCommand {
    pub text: String,
    pub script: Option<Script>,
}

impl Command for BraceCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) {
        let script = match self.script {
            Some(ref mut s) => s,
            _ => panic!("SUSH INTERNAL ERROR (BraceCommand::exec)"),
        };

        if pipe.is_connected() {
            script.fork_exec(core, pipe);
        }else{
            script.exec(core);
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl BraceCommand {
    fn new() -> BraceCommand {
        BraceCommand {
            text: String::new(),
            script: None,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BraceCommand> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "{", &mut ans.script) {
            ans.text = "{".to_string() + &ans.script.as_mut().unwrap().text.clone() + &feeder.consume(1);
            Some(ans)
        }else{
            None
        }
    }
}
