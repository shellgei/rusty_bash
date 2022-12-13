//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::{env, process};
use std::path::Path;

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.vars.insert("?".to_string(), args[1].clone());
    }

    let exit_status = match core.vars["?"].parse::<i32>() {
        Ok(n)  => n%256, 
        Err(_) => {
            eprintln!("sush: exit: {}: numeric argument required", core.vars["?"]);
            2
        },
    };

    process::exit(exit_status)
}

pub fn cd(_: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() > 1 {
        let path = Path::new(&args[1]);
        if env::set_current_dir(&path).is_err() {
            eprintln!("Cannot change directory");
            return 1;
        }

    }
    0
}
