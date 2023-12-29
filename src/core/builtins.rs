//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::{env, fs};
use std::path::Path;

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("cd".to_string(), cd);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("pwd".to_string(), pwd);
        self.builtins.insert("true".to_string(), true_);
    }
}

pub fn cd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 0 {
        eprintln!("SUSH INTERNAL ERROR: (no arg for cd)");
        return 1;
    }
    if args.len() > 2 {
        eprintln!("sush: cd: too many arguments");
        return 1;
    }

    if args.len() == 1 { //only "cd"
        let var = env::var("HOME").expect("HOME is not defined");
        args.push(var);
    }else if args.len() == 2 && args[1] == "-" { // cd -
        if let Some(old) = core.vars.get("OLDPWD") {
            println!("{}", &old);
            args[1] = old.to_string();
        }
    };

    if let Ok(old) = env::current_dir() {
        core.vars.insert("OLDPWD".to_string(), old.display().to_string());
    };

    let path = Path::new(&args[1]);
    if env::set_current_dir(&path).is_ok() {
        if let Ok(full) = fs::canonicalize(path) {
            core.vars.insert("PWD".to_string(), full.display().to_string());
        }
        0
    }else{
        eprintln!("sush: cd: {:?}: No such file or directory", &path);
        1
    }
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.vars.insert("?".to_string(), args[1].clone());
    }
    core.exit()
}

pub fn false_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    1
}

pub fn pwd(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    match env::current_dir() {
        Ok(path) => {
            println!("{}", path.display());
            0
        },
        Err(err) => {
            eprintln!("pwd: error retrieving current directory: {:?}", err);
            1
        }
    }
}

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}
