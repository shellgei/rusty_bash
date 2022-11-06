//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use nix::unistd::execvp;
use std::ffi::CString;
use std::process;

use nix::unistd::{fork, ForkResult}; 

pub struct Command {
    text: String,
    args: Vec<String>,
    cargs: Vec<CString>,
}

impl Command {
    pub fn exec(&mut self, _core: &mut ShellCore) {
        if self.text == "exit\n" {
            process::exit(0);
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    eprintln!("コマンド実行");
                    return;
                },
                Ok(ForkResult::Parent { child } ) => {
                    eprintln!("PID{}の親です", child);
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
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
            Some( Command {text: line, args: args, cargs: cargs} )
        }else{
            None
        }
    }
}
