//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct WhileCommand {
    pub text: String,
    pub condition: Option<Script>,
    pub inner: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for WhileCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        self.nofork_exec(core, pipe)
    }

    fn get_text(&self) -> String { self.text.clone() }

    fn set_force_fork(&mut self) {
        self.force_fork = true;
    }
}

impl WhileCommand {
    fn nofork_exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        loop {
            self.condition.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core, &mut vec![]);

            if core.vars["?"] != "0" {
                break;
            }

            self.inner.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core, &mut vec![]);
        }
        None
    }

    fn new() -> WhileCommand {
        WhileCommand {
            text: String::new(),
            condition: None,
            inner: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<WhileCommand> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "while", vec!["do"], &mut ans.condition)
        && command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.inner) {
            ans.text.push_str("while");
            ans.text.push_str(&ans.condition.as_mut().unwrap().text.clone());
            ans.text.push_str("do");
            ans.text.push_str(&ans.inner.as_mut().unwrap().text.clone());
            ans.text.push_str(&feeder.consume(4)); //done
            //TODO: eat redirect
            Some(ans)
        }else{
            None
        }
    }
}
