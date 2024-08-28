//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::command;

#[derive(Debug, Clone)]
pub struct TestCommand {
    text: String,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for TestCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        core.data.set_param("?", "0");
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl TestCommand {
    fn new() -> TestCommand {
        TestCommand {
            text: String::new(),
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[[") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        while ! feeder.starts_with("]]") {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }
    
        command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
        None
    }
}
