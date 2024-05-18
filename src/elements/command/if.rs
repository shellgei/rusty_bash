//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Redirect};

#[derive(Debug)]
pub struct IfCommand {
    pub text: String,
    pub if_elif_scripts: Vec<Script>,
    pub then_scripts: Vec<Script>,
    pub else_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for IfCommand {
    fn run(&mut self, _: &mut ShellCore, _: bool) {}
    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn force_fork(&self) -> bool { self.force_fork }
}

impl IfCommand {
    fn new() -> IfCommand {
        IfCommand {
            text: String::new(),
            if_elif_scripts: vec![],
            then_scripts: vec![],
            else_script: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(_: &mut Feeder, _: &mut ShellCore) -> Option<IfCommand> {None}
}
