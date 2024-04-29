//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod cd;
mod pwd;
mod utils;

use crate::{Feeder, Script, ShellCore};
use crate::elements::io;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::fd::IntoRawFd;

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("set".to_string(), set);
        self.builtins.insert("source".to_string(), source);
        self.builtins.insert(".".to_string(), source);
        self.builtins.insert("true".to_string(), true_);
    }
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.parameters.insert("?".to_string(), args[1].clone());
    }
    core.exit()
}

pub fn false_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    1
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    core.position_parameters = args.to_vec();
    0
}

pub fn source(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        eprintln!("sush: source: filename argument required");
        eprintln!("source: usage: source filename [arguments]");
        return 2;
    }
    let file = match File::open(&args[1]) {
        Ok(f) => f, 
        _     => return 1, 
    };

    let fd = file.into_raw_fd();
    let backup = io::backup(0);
    io::replace(fd, 0);
    core.flags = core.flags.replace("i", "@");

    let mut feeder = Feeder::new();
    loop {
        match feeder.feed_line(core) {
            Ok(()) => {eprintln!("{:?}", &feeder);}, 
            _ => break,
        }

        match Script::parse(&mut feeder, core, false){
            Some(mut s) => s.exec(core),
            None => {},
        }
    }

    core.flags = core.flags.replace("@", "i");
    io::replace(backup, 0);
    0
}

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}
