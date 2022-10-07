//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use std::process;

pub struct Command {
    pub text: String,
}

impl Command {
    pub fn exec(&mut self, _core: &mut ShellCore) { //引数_coreはまだ使いません
        if self.text == "exit\n" {
            process::exit(0);
        }

        let mut words = vec![]; //ベクタを作る
        for w in self.text.trim_end().split(' ') { //空白で分割
            words.push(w);
        };

        println!("{:?}", words);
    }

    pub fn parse(feeder: &mut Feeder, _core: &mut ShellCore) -> Option<Command> {
        let line = feeder.consume(feeder.remaining.len());
        Some( Command {text: line} )
    }
}
