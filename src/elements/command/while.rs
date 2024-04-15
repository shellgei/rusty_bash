//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug, Clone)]
pub struct WhileCommand {
    pub text: String,
    pub while_script: Option<Script>,
    pub do_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for WhileCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork || pipe.is_connected() {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn run(&mut self, core: &mut ShellCore, _: bool) {
        loop {
            self.while_script.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core);

            if core.get_param_ref("?") != "0" {
                break;
            }

            self.do_script.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core);
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
}

impl WhileCommand {
    fn new() -> WhileCommand {
        WhileCommand {
            text: String::new(),
            while_script: None,
            do_script: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<WhileCommand> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "while", vec!["do"], &mut ans.while_script)
        && command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.do_script) {
            ans.text.push_str("while");
            ans.text.push_str(&ans.while_script.as_mut().unwrap().get_text());
            ans.text.push_str("do");
            ans.text.push_str(&ans.do_script.as_mut().unwrap().get_text());
            ans.text.push_str(&feeder.consume(4)); //done

            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            Some(ans)
        }else{
            None
        }
    }
}
