//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder,Script};
use super::Command;
use nix::unistd;
use nix::unistd::ForkResult;

#[derive(Debug)]
pub struct ParenCommand {
    pub text: String,
    pub script: Option<Script>,
}

impl Command for ParenCommand {
    fn exec(&mut self, core: &mut ShellCore) {
        match self.script {
            Some(ref mut s) => Self::fork_exec(s, core),
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

    fn fork_exec(script: &mut Script, core: &mut ShellCore) {
        //eprintln!("fork前: {}", &core.vars["BASHPID"]);
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                let pid = nix::unistd::getpid();
                core.vars.insert("BASHPID".to_string(), pid.to_string());
                //eprintln!("fork後: {}", &core.vars["BASHPID"]);
                script.exec(core);
                core.exit();
            },
            Ok(ForkResult::Parent { child } ) => {
                core.wait_process(child);
            },
            Err(err) => panic!("Failed to fork. {}", err),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ParenCommand> {
        match Script::parse_with_left(feeder, core, "(") {
            Some(s) => {
                let mut ans = Self::new();
                ans.text = s.text.clone() + &feeder.consume(1);
                ans.script = Some(s);
                Some(ans)
            },
            None => None, 
        }
    }
}
