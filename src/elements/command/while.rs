//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::elements::Pipe;
use crate::elements::command;
use crate::elements::command::Command;
use crate::elements::io::redirect::Redirect;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct WhileCommand {
    pub text: String,
    pub while_script: Option<Script>,
    pub do_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for WhileCommand {
    fn exec(&mut self, _: &mut ShellCore, _: &mut Pipe) -> Option<Pid> {
        None
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn set_force_fork(&mut self) { self.force_fork = true; }
}

impl WhileCommand {
    fn new() -> WhileCommand {
        WhileCommand {
            text: String::new(),
            while_script: None,
            do_script: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<WhileCommand> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "while", vec!["do"], &mut ans.while_script)
        && command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.do_script) {
            ans.text.push_str("while");
            ans.text.push_str(&ans.while_script.as_mut().unwrap().text.clone());
            ans.text.push_str("do");
            ans.text.push_str(&ans.do_script.as_mut().unwrap().text.clone());
            ans.text.push_str(&feeder.consume(4)); //done

            loop {
                command::eat_blank_with_comment(feeder, core, &mut ans.text);
                if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text){
                    break;
                }
            }
            dbg!("{:?}", &ans);
            Some(ans)
        }else{
            None
        }
    }
}
