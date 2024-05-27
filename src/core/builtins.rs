//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod cd;
pub mod completion;
mod local;
mod pwd;
mod read;
pub mod set;
mod source;
mod return_break;
mod utils;

use crate::ShellCore;

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("alias".to_string(), alias);
        self.builtins.insert("break".to_string(), return_break::break_);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("compgen".to_string(), completion::compgen);
        self.builtins.insert("complete".to_string(), completion::complete);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("local".to_string(), local::local);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("read".to_string(), read::read);
        self.builtins.insert("return".to_string(), return_break::return_);
        self.builtins.insert("set".to_string(), set::set);
        self.builtins.insert("source".to_string(), source::source);
        self.builtins.insert(".".to_string(), source::source);
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

/*
pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match core.data.position_parameters.pop() {
        None => panic!("SUSH INTERNAL ERROR: empty param stack"),
        _    => {},
    }
    core.data.position_parameters.push(args.to_vec());
    0
}*/

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}
