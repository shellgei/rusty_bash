//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct BraceCommand {
    pub text: String,
    pub script: Option<Script>,
    pub redirects: Vec<Redirect>,
}

impl Command for BraceCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        let script = match self.script {
            Some(ref mut s) => s,
            _ => panic!("SUSH INTERNAL ERROR (BraceCommand::exec)"),
        };

        if pipe.is_connected() {
            script.fork_exec(core, pipe, &mut self.redirects)
        }else{
            script.exec(core);
            None
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl BraceCommand {
    fn new() -> BraceCommand {
        BraceCommand {
            text: String::new(),
            script: None,
            redirects: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BraceCommand> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "{", &mut ans.script) {
            ans.text = "{".to_string() + &ans.script.as_mut().unwrap().text.clone() + &feeder.consume(1);

            loop {
                command::eat_blank_with_comment(feeder, core, &mut ans.text);
                if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text){
                    break;
                }
            }

//            eprintln!("{:?}", ans);
            Some(ans)
        }else{
            None
        }
    }
}
