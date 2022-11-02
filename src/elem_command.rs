//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use nix::unistd::execvp;
use std::ffi::CString;
use std::process;

pub struct Command {
    pub text: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn exec(&mut self, _core: &mut ShellCore) {
        if self.text == "exit\n" {
            process::exit(0);
        }

        if self.args.len() > 0 {
            let cwords: Vec<CString> = self.args
                .iter()
                .map(|w| CString::new(w.clone()).unwrap())
                .collect();
            println!("{:?}", execvp(&cwords[0], &cwords));
        }
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        let args = line
            .trim_end()
            .split(' ')
            .map(|w| w.to_string())
            .collect();

        Some( Command {text: line, args: args} )
    }
}
