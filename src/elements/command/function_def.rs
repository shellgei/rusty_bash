//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::{Command, Redirect};

#[derive(Debug, Clone, Default)]
pub struct FunctionDefinition {
    pub text: String,
    name: String,
    command: Option<Box<dyn Command>>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for FunctionDefinition {
    fn run(&mut self, _: &mut ShellCore, _: bool) -> Result<(), ExecError> {Ok(())}
    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn force_fork(&self) -> bool { self.force_fork }
}

impl FunctionDefinition {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        let has_function_keyword = feeder.starts_with("function");
        if has_function_keyword {
            ans.text += &feeder.consume(8);
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }

        if ! Self::eat_name(feeder, &mut ans, core) {
            feeder.rewind();
            return Ok(None);
        }


    }
}
