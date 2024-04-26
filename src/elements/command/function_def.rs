//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    text: String,
    name: String,
    script: Option<Box<dyn Command>>,
    redirects: Vec<Redirect>,
}

impl Command for FunctionDefinition {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        None
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) { }
    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
}

impl FunctionDefinition {
    fn new() -> FunctionDefinition {
        FunctionDefinition {
            text: String::new(),
            name: String::new(),
            script: None,
            redirects: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore, permit_empty: bool) -> Option<Self> {
        None
    }
}
