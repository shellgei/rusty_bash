//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use nix::unistd::execvp;
use std::ffi::CString;
use std::process;

use nix::unistd::{fork, ForkResult, Pid}; //追加
use std::process::exit;

pub struct Command {
    pub text: String,
    pub args: Vec<String>,
    pub cargs: Vec<CString>,
    pub pid: Option<Pid>,
}

impl Command {
    pub fn exec(&mut self, _core: &mut ShellCore) {
        if self.text == "exit\n" {
            process::exit(0);
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    let _ = execvp(&self.cargs[0], &self.cargs);
                    println!("Command not found");
                    exit(127);
                },
                Ok(ForkResult::Parent { child } ) => {
                    self.pid = Some(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        eprintln!("LINE: {}", line);
        let args: Vec<String> = line
            .trim_end()
            .split(' ')
            .map(|w| w.to_string())
            .collect();

        let cargs: Vec<CString> = args
            .iter()
            .map(|w| CString::new(w.clone()).unwrap())
            .collect();

        if args.len() > 0 {
            Some( Command {text: line, args: args, cargs: cargs, pid: None} )
        }else{
            None
        }
    }
}
