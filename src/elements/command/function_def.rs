//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use std::collections::HashMap;
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
    text: String,
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

        core.data.functions.insert(self.name.to_string(), self.clone());
        None
    }

    fn run(&mut self, _: &mut ShellCore, _: bool) { }
    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
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
                          core: &mut ShellCore, pipe: Option<&mut Pipe>) -> Option<Pid> {
        //let backup = core.data.position_parameters.to_vec();
        //args[0] = core.data.position_parameters[0].clone();
        //core.data.position_parameters = args.to_vec();

        let len = core.data.position_parameters.len();
        args[0] = core.data.position_parameters[len-1][0].clone();
        core.data.position_parameters.push(args.to_vec());
        core.data.local_parameters.push(HashMap::new());
        let mut empty = Pipe::new("|".to_string());
        let p = match pipe {
            Some(p) => p,
            _       => &mut empty,
        };

        let pid = self.command.clone()
                        .expect("SUSH INTERNAL ERROR: empty function")
                        .exec(core, p);

        core.data.position_parameters.pop();
        core.data.local_parameters.pop();

        return pid;
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        ans.name = feeder.consume(len).to_string();

        if ans.name.len() == 0 && reserved(&ans.name) {
            return false;
        }
        ans.text += &ans.name;
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        true
    }

    fn eat_compound_command(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        ans.command = if let Some(a) = IfCommand::parse(feeder, core) { Some(Box::new(a)) }
        else if let Some(a) = ParenCommand::parse(feeder, core, false) { Some(Box::new(a)) }
        else if let Some(a) = BraceCommand::parse(feeder, core) { Some(Box::new(a)) }
        else if let Some(a) = WhileCommand::parse(feeder, core) { Some(Box::new(a)) }
        else {None};

        if let Some(c) = &ans.command {
            ans.text += &c.get_text();
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let mut ans = Self::new();
        feeder.set_backup();

        if feeder.starts_with("function") {
            ans.text += &feeder.consume(8);
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }
        
        if ! Self::eat_name(feeder, &mut ans, core) 
        || ! feeder.starts_with("()") {
            feeder.rewind();
            return None;
        }
        ans.text += &feeder.consume(2);
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        Self::eat_compound_command(feeder, &mut ans, core);
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        if let Some(_) = &ans.command {
            feeder.pop_backup();
            Some(ans)
        }else{
            feeder.rewind();
            None
        }
    }
}
