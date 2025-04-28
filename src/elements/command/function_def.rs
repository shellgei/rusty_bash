//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
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
    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        ans.name = feeder.consume(len).to_string();

        if ans.name.is_empty() && utils::reserved(&ans.name) {
            return false;
        }
        ans.text += &ans.name;
        super::eat_blank_with_comment(feeder, core, &mut ans.text);

        true
    }

    fn eat_compound_command(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        ans.command = if let Some(a) = IfCommand::parse(feeder, core)? { Some(Box::new(a)) }
        else if let Some(a) = ParenCommand::parse(feeder, core, false)? { Some(Box::new(a)) }
        else if let Some(a) = BraceCommand::parse(feeder, core)? { Some(Box::new(a)) }
        else if let Some(a) = WhileCommand::parse(feeder, core)? { Some(Box::new(a)) }
        else {None};
 
        if let Some(c) = &ans.command {
            ans.text += &c.get_text();
            Ok(true)
        }else{
            Ok(false)
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        return Ok(None);
    }
}
