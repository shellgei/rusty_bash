//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod cd;
pub mod completion;
mod pwd;
mod utils;

use crate::{Feeder, Script, ShellCore};
use crate::elements::io;
use crate::elements::substitution::{Substitution, Value};
use std::fs::File;
use std::os::fd::IntoRawFd;
use std::path::Path;

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("alias".to_string(), alias);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("compgen".to_string(), completion::compgen);
        self.builtins.insert("complete".to_string(), completion::complete);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("local".to_string(), local);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("return".to_string(), return_);
        self.builtins.insert("set".to_string(), set);
        self.builtins.insert("source".to_string(), source);
        self.builtins.insert(".".to_string(), source);
        self.builtins.insert("true".to_string(), true_);
    }
}

pub fn alias(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
        for (k, v) in &core.data.aliases {
            println!("alias {}='{}'", k, v);
        }
        return 0;
    }

    if args.len() == 2 && args[1].find("=") != None {
        let kv: Vec<String> = args[1].split("=").map(|t| t.to_string()).collect();
        core.data.aliases.insert(kv[0].clone(), kv[1..].join("="));
    }

    0
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.data.parameters[0].insert("?".to_string(), args[1].clone());
    }
    core.exit()
}

pub fn false_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    1
}

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if core.data.parameters.len() <= 2 {
        eprintln!("sush: local: can only be used in a function");
        return 1;
    }

    let layer = core.data.parameters.len() - 2; //The last element of data.parameters is for local itself.

    for arg in &args[1..] {
        let mut feeder = Feeder::new();
        feeder.add_line(arg.clone());
        match Substitution::parse(&mut feeder, core) {
            Some(mut sub) => {
                match sub.eval(core) {
                    Value::EvaluatedSingle(s) => {
                        core.data.parameters[layer].insert(sub.key.to_string(), s);
                    },
                    Value::EvaluatedArray(a) => {
                        core.data.arrays[layer].insert(sub.key.to_string(), a);
                    },
                    _ => {
                        eprintln!("sush: local: `{}': not a valid identifier", arg);
                        return 1;
                    },
                }
            },
            _ => {
                eprintln!("sush: local: `{}': not a valid identifier", arg);
                return 1;
            },
        }
    }

    0
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let len = core.data.position_parameters.len();

    if len == 0 {
        panic!("SUSH INTERNAL ERROR: empty param stack");
    }

    core.data.position_parameters[len-1].clear();
    core.data.position_parameters[len-1].append(args);
    0
}

pub fn source(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        eprintln!("sush: source: filename argument required");
        eprintln!("source: usage: source filename [arguments]");
        return 2;
    }

    if Path::new(&args[1]).is_dir() {
        eprintln!("bash: source: {}: is a directory", &args[1]);
        return 1;
    }

    let file = match File::open(&args[1]) {
        Ok(f)  => f, 
        Err(e) => {
            eprintln!("sush: {}: {}", &args[1], &e);
            return 1;
        }, 
    };

    let fd = file.into_raw_fd();
    let backup = io::backup(0);
    io::replace(fd, 0);
    core.in_source = true;

    let mut feeder = Feeder::new();
    loop {
        match feeder.feed_line(core) {
            Ok(()) => {}, 
            _ => break,
        }

        if core.return_flag {
            feeder.consume(feeder.len());
        }

        match Script::parse(&mut feeder, core, false){
            Some(mut s) => s.exec(core),
            None => {},
        }
    }


    io::replace(backup, 0);
    core.in_source = false;
    core.return_flag = false;
    core.data.get_param("?").parse::<i32>()
        .expect("SUSH INTERNAL ERROR: BAD EXIT STATUS")
}

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}

pub fn return_(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    core.return_flag = true;
    0
}
