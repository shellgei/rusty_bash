//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use super::command;
use super::command::Command;

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<Box<dyn Command>>,
    pub pipes: Vec<String>,
    pub text: String,
}

impl Pipeline {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for command in self.commands.iter_mut() {
            command.exec(core);
        }
    }

    pub fn new() -> Pipeline {
        Pipeline {
            text: String::new(),
            commands: vec![],
            pipes: vec![]
        }
    }

    fn eat_command(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Pipeline) -> bool {
        if let Some(command) = command::parse(feeder, core){
            ans.text += &command.get_text();
            ans.commands.push(command);
            true
        }else{
            false
        }
    }

    fn eat_pipe(feeder: &mut Feeder, ans: &mut Pipeline) -> bool {
        let len = feeder.scanner_pipe();
        if len > 0 {
            let p = feeder.consume(len);
            ans.pipes.push(p.clone());
            ans.text += &p;
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();

        while Self::eat_command(feeder, core, &mut ans)
              && Self::eat_pipe(feeder, &mut ans){
        }

        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
