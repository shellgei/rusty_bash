//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::io::redirect::Redirect;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct WhileCommand {
    pub text: String,
    pub while_script: Option<Script>,
    pub do_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for WhileCommand {
    fn exec(&mut self, _: &mut ShellCore, _: &mut Pipe) -> Option<Pid> {
        None
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn set_force_fork(&mut self) { self.force_fork = true; }
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

    pub fn parse(_: &mut Feeder, _: &mut ShellCore) -> Option<WhileCommand> {
        None
    }
}
