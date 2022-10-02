//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use nix::unistd::execvp;
use std::process;
use std::ffi::CString;

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
            .map(|a| CString::new(a.to_string()).unwrap()) //文字列を一つずつCString型に変換
            .collect();

        println!("{:?}", words);
        if words.len() > 0 {
            println!("{:?}", execvp(&words[0], &*words));
        }
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        Some( Command {text: line} )
    }
}
