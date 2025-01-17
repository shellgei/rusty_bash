//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::command::{BraceCommand, IfCommand, ParenCommand, WhileCommand};
use nix::unistd::Pid;

fn reserved(w: &str) -> bool {
    match w {
        "{" | "}" | "while" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi" => true,
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub text: String,
    name: String,
    command: Option<Box<dyn Command>>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for FunctionDefinition {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork || pipe.is_connected() {
            return None;
        }

        core.db.functions.insert(self.name.to_string(), self.clone());
        None
    }

    fn run(&mut self, _: &mut ShellCore, _: bool) { }
    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl FunctionDefinition {
    fn new() -> FunctionDefinition {
        FunctionDefinition {
            text: String::new(),
            name: String::new(),
            command: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn run_as_command(&mut self, args: &mut Vec<String>,
                          core: &mut ShellCore) -> Option<Pid> {
        let mut array = core.db.get_array_all("FUNCNAME");
        array.insert(0, args[0].clone());
        let _ = core.db.set_array("FUNCNAME", array, None);

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

        //core.db.set_param("#", &number);, None, None
        let mut array = core.db.get_array_all("FUNCNAME");
        array.remove(0);
        let _ = core.db.set_array("FUNCNAME", array, None);
        return pid;
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        ans.name = feeder.consume(len).to_string();

        if ans.name.is_empty() && reserved(&ans.name) {
            return false;
        }
        ans.text += &ans.name;
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        true
    }

    fn eat_compound_command(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        ans.command = if let Some(a) = IfCommand::parse(feeder, core) { Some(Box::new(a)) }
        else if let Some(a) = ParenCommand::parse(feeder, core, false) { Some(Box::new(a)) }
        else if let Some(a) = BraceCommand::parse(feeder, core) { Some(Box::new(a)) }
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
        let mut ans = Self::new();
        feeder.set_backup();

        if feeder.starts_with("function") {
            ans.text += &feeder.consume(8);
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }
        
        if ! Self::eat_name(feeder, &mut ans, core) 
        || ! feeder.starts_with("()") {
            feeder.rewind();
            return Ok(None);
        }
        ans.text += &feeder.consume(2);
        loop {
            if feeder.starts_with("\n") {
                ans.text += &feeder.consume(1);
                continue;
            }

            if feeder.len() == 0 {
                if ! feeder.feed_additional_line(core).is_ok() {
                    return Ok(None);
                }
            }
            if ! command::eat_blank_with_comment(feeder, core, &mut ans.text) {
                break;
            }
        }

        Self::eat_compound_command(feeder, &mut ans, core)?;
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        if let Some(_) = &ans.command {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind();
            Ok(None)
        }
    }
}
