//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Pipe};
use crate::elements::command;
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
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) {
        if ! pipe.is_connected() && core.run_builtin(&mut self.args) {
            return;
        }

        self.set_cargs();
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                pipe.connect();
                if core.run_builtin(&mut self.args) {
                    core.exit();
                }

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
                pipe.parent_close();
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
            cargs: vec![],
        }
    }
 
    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore) -> bool {
        let arg_len = feeder.scanner_word(core);
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

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleCommand> {
        let mut ans = Self::new();
        let backup = feeder.clone();

        command::eat_blank_with_comment(feeder, &mut ans.text, core);
        while Self::eat_word(feeder, &mut ans, core) &&
              command::eat_blank_with_comment(feeder, &mut ans.text, core) {}

        if ans.args.len() > 0 {
            Some(ans)
        }else{
            feeder.rewind(backup);
            None
        }
    }
}
