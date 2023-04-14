//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use super::Command;
use nix::unistd;
use std::ffi::CString;
use std::process;

use nix::unistd::ForkResult;
use nix::errno::Errno;

#[derive(Debug)]
pub struct SimpleCommand {
    pub text: String,
    args: Vec<String>,
    cargs: Vec<CString>,
}

impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore) {
        if core.run_builtin(&mut self.args) {
            return;
        }

        self.set_cargs();
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                match unistd::execvp(&self.cargs[0], &self.cargs) {
                    Err(Errno::EACCES) => {
                        println!("sush: {}: Permission denied", &self.args[0]);
                        process::exit(126)
                    },
                    Err(Errno::ENOENT) => {
                        println!("{}: command not found", &self.args[0]);
                        process::exit(127)
                    },
                    Err(err) => {
                        println!("Failed to execute. {:?}", err);
                        process::exit(127)
                    }
                    _ => ()
                }
            },
            Ok(ForkResult::Parent { child } ) => {
                core.wait_process(child);
            },
            Err(err) => panic!("Failed to fork. {}", err),
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl SimpleCommand {
    fn set_cargs(&mut self) {
        self.cargs = self.args.iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();
    }

   fn new() -> SimpleCommand {
       SimpleCommand {
           text: String::new(),
           args: vec![],
           cargs: vec![]
       }
   }

   fn eat_blank(feeder: &mut Feeder, ans: &mut SimpleCommand) -> bool {
       let blank_len = feeder.scanner_blank();
       if blank_len == 0 {
           return false;
       }
       ans.text += &feeder.consume(blank_len);
       true
   }

   fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand) -> bool {
       let arg_len = feeder.scanner_word();
       if arg_len == 0 {
           return false;
       }

       let word = feeder.consume(arg_len);
       if ans.args.len() == 0 && ( word == "{" || word == "}") {
           return false;
       }

       ans.text += &word.clone();
       ans.args.push(word);
       true
   }

    pub fn parse(feeder: &mut Feeder, _: &mut ShellCore) -> Option<SimpleCommand> {
        let mut ans = Self::new();
        let backup = feeder.clone();

        Self::eat_blank(feeder, &mut ans);
        while Self::eat_word(feeder, &mut ans) &&
              Self::eat_blank(feeder, &mut ans) {}

        if ans.args.len() > 0 {
            Some(ans)
        }else{
            feeder.rewind(backup);
            None
        }
    }
}
