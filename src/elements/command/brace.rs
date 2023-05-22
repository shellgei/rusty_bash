//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script, Pipe};
use super::Command;

#[derive(Debug)]
pub struct BraceCommand {
    pub text: String,
    pub script: Option<Script>,
    pipe: Pipe,
}

impl Command for BraceCommand {
    fn exec(&mut self, core: &mut ShellCore) {
        let script = match self.script {
            Some(ref mut s) => s,
            _ => panic!("SUSH INTERNAL ERROR (BraceCommand::exec)"),
        };
         
        if self.pipe.is_connected() {
            script.fork_exec(core, &mut self.pipe);
        }else{
            script.exec(core);
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn set_pipe(&mut self, pipe: Pipe){ self.pipe = pipe; }
}

impl BraceCommand {
    fn new() -> BraceCommand {
        BraceCommand {
            text: String::new(),
            script: None,
            pipe: Pipe::new(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BraceCommand> {
        match Script::parse_nested(feeder, core, "{") {
            Some(s) => {
                let mut ans = Self::new();
                ans.text = "{".to_string() + &s.text.clone() + &feeder.consume(1);
                ans.script = Some(s);
                Some(ans)
            },
            None => None, 
        }
    }
}
