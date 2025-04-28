//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils;
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::command::{BraceCommand, IfCommand, ParenCommand, WhileCommand};
use nix::unistd::Pid;

#[derive(Debug, Clone, Default)]
pub struct FunctionDefinition {
    pub text: String,
    pub file: String,
    name: String,
    command: Option<Box<dyn Command>>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for FunctionDefinition {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        if self.force_fork || pipe.is_connected() {
            return Ok(None);
        }

        core.db.functions.insert(self.name.to_string(), self.clone());
        Ok(None)
    }

    fn run(&mut self, _: &mut ShellCore, _: bool) -> Result<(), ExecError> {Ok(())}
    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }

    fn pretty_print(&mut self, indent_num: usize) {
        self.pretty_print(indent_num);
    }
}

impl FunctionDefinition {
    pub fn pretty_print(&mut self, indent_num: usize) {
        println!("{} () ", self.name);
        for com in self.command.iter_mut() {
            com.pretty_print(indent_num);
        }
    }

    pub fn run_as_command(&mut self, args: &mut Vec<String>, core: &mut ShellCore)
        -> Result<Option<Pid>, ExecError> {
        let mut array = core.db.get_array_all("FUNCNAME");
        array.insert(0, args[0].clone()); //TODO: We must put the name not only in 0 but also 1..
        let _ = core.db.set_array("FUNCNAME", array.clone(), None);
        let mut source = core.db.get_array_all("BASH_SOURCE");
        source.insert(0, self.file.clone());
        let _ = core.db.set_array("BASH_SOURCE", source.clone(), None);

        let len = core.db.position_parameters.len();
        args[0] = core.db.position_parameters[len-1][0].clone();
        core.db.position_parameters.push(args.to_vec());

        let mut dummy = Pipe::new("|".to_string());

        core.source_function_level += 1;
        let pid = self.command.clone()
                        .unwrap()
                        .exec(core, &mut dummy);
        core.return_flag = false;
        core.source_function_level -= 1;

        core.db.position_parameters.pop();

        array.remove(0);
        source.remove(0);
        let _ = core.db.set_array("FUNCNAME", array, None);
        let _ = core.db.set_array("BASH_SOURCE", source, None);
        pid
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        ans.name = feeder.consume(len).to_string();

        if ans.name.is_empty() && utils::reserved(&ans.name) {
            return false;
        }
        ans.text += &ans.name;
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

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

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
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

        if feeder.starts_with("()") {
            ans.text += &feeder.consume(2);
        }else if ! has_function_keyword {
            feeder.rewind();
            return Ok(None);
        }
        
        let _ = command::eat_blank_lines(feeder, core, &mut ans.text);
        let result = Self::eat_compound_command(feeder, &mut ans, core);

        if ans.command.is_some() {
            feeder.pop_backup();
            if let Some(f) = core.source_files.last() {
                ans.file = f.clone();
            }
            Ok(Some(ans))
        }else{
            feeder.rewind();
            result?;
            Ok(None)
        }
    }
}
