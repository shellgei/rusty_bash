//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::process;
use std::ffi::CString;
use crate::{ShellCore,Feeder};

pub struct Command {
    pub text: String,
}

impl Command {
    pub fn exec(&mut self, _core: &mut ShellCore) { //引数_coreはまだ使いません
        if self.text == "exit\n" {
            process::exit(0);
        }

        let words: Vec<CString> = self.text
            .trim_end() //末尾の改行（'\n'）を削除
            .split(' ') //半角スペースで分割
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        println!("{:?}", words);
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        Some( Command {text: line} )
    }
}
