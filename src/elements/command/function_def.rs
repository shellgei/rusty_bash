//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
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

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        eprintln!("here {:?}", &feeder);
        let mut ans = Self::new();
        feeder.set_backup();
        
        if ! Self::eat_name(feeder, &mut ans, core) 
        || feeder.starts_with("()") {
            feeder.rewind();
            return None;
        }

        dbg!("{:?}", &ans);


        feeder.rewind();
        None
    }
}
