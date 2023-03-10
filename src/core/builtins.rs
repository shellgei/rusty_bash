//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::{env, fs};
use std::path::Path;

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.vars.insert("?".to_string(), args[1].clone());
    }
    core.exit()
}

pub fn cd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 0 {
        eprintln!("Bug of this shell");
        return 1;
    }
    if args.len() > 2 {
        eprintln!("{}", "bash: cd: too many arguments");
        return 1;
    }


    if args.len() == 1 { //only "cd"
        let var = env::var("HOME").expect("HOME is not defined");
        args.push(var);
    }else if args.len() == 2 && args[1] == "-" { // cd -
        if let Some(old) = core.vars.get("OLDPWD") {
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
        eprintln!("Not exist directory");
        1
    }
}
