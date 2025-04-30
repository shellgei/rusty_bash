//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, utils};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::{Command, Redirect};
use super::{BraceCommand, IfCommand, ParenCommand, WhileCommand};

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
    fn eat_header(&mut self, feeder: &mut Feeder, core: &mut ShellCore) -> bool {
        let has_function_keyword = feeder.starts_with("function");
        if has_function_keyword {
            self.text += &feeder.consume(8);
            command::eat_blank_with_comment(feeder, core, &mut self.text);
        }

        let len = feeder.scanner_name(core);
        self.name = feeder.consume(len).to_string();

        if self.name.is_empty() && utils::reserved(&self.name) {
            return false;
        }
        self.text += &self.name;
        command::eat_blank_with_comment(feeder, core, &mut self.text);

        if feeder.starts_with("()") {
            self.text += &feeder.consume(2);
        }else if ! has_function_keyword {
            return false;
        }

        let _ = command::eat_blank_lines(feeder, core, &mut self.text);
        true
    } 



    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        return Ok(None);
    }
}
