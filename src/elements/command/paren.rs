//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{utils::error, ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug, Clone)]
pub struct ParenCommand {
    text: String,
    script: Option<Script>,
    redirects: Vec<Redirect>,
}

impl Command for ParenCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        self.fork_exec(core, pipe)
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) {
        if ! fork {
            error::internal(" (no fork for subshell)");
        }

        match self.script {
            Some(ref mut s) => s.exec(core),
            _ => error::internal(" (ParenCommand::exec)"),
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { true }
}

impl ParenCommand {
    fn new() -> ParenCommand {
        ParenCommand {
            text: String::new(),
            script: None,
            redirects: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore, substitution: bool) -> Option<Self> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "(", vec![")"], &mut ans.script, substitution) {
            ans.text.push_str("(");
            ans.text.push_str(&ans.script.as_ref().unwrap().get_text());
            ans.text.push_str(&feeder.consume(1));

            if ! substitution {
                command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            }
            Some(ans)
        }else{
            None
        }
    }
}
