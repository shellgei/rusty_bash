//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
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

impl SimpleCommand {
    pub fn exec(&mut self, core: &mut ShellCore) {
        if core.run_builtin(&mut self.args) {
            return;
        }

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

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<SimpleCommand> {
        let mut ans = SimpleCommand { text: String::new(), args: vec![], cargs: vec![] };
        let backup = feeder.clone();

        let blank_len = feeder.scanner_blank();
        ans.text += &feeder.consume(blank_len);

        loop {
            let arg_len = feeder.scanner_word();
            if arg_len == 0 {
                break;
            }
            let word = feeder.consume(arg_len);
            ans.text += &word.clone();
            ans.args.push(word);

            let blank_len = feeder.scanner_blank();
            if blank_len == 0 {
                break;
            }
            ans.text += &feeder.consume(blank_len);
        }

        eprintln!("{:?}", ans);
        None
    }
}
