//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore, Pipe};
use nix::unistd;
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
        self.pipes.resize(self.commands.len(), "".to_string());

        let mut prev_out = -1;
        for (i, command) in self.commands.iter_mut().enumerate() {
            let p = match self.pipes[i].as_ref() {
                "" => (-1, -1),
                _  => unistd::pipe().expect("Pipe cannot open"),
            };

            let mut pinfo = Pipe{my_in: p.0, my_out: p.1, prev_out: prev_out};
            command.exec(core, &mut pinfo);

            if p.1 >= 0 { 
                unistd::close(p.1).expect("Cannot close parent pipe out");
            }
            prev_out = p.0;
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

            let blank_len = feeder.scanner_blank();
            ans.text += &feeder.consume(blank_len);
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

            let blank_len = feeder.scanner_blank();
            ans.text += &feeder.consume(blank_len);
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();

        while Self::eat_command(feeder, core, &mut ans)
              && Self::eat_pipe(feeder, &mut ans){ }

        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
