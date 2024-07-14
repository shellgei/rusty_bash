//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn shopt_print(core: &mut ShellCore, args: &mut Vec<String>, all: bool) -> i32 {
    if all {
        core.shopts.print_all();
        return 0;
    }

    let mut res = true;
    match args[1].as_str() {
        "-s" => core.shopts.print_if(true),
        "-u" => core.shopts.print_if(false),
        opt  => res = core.shopts.print_opt(opt),
    }

    match res {
        true  => 0,
        false => 1,
    }
}

pub fn shopt(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 3 {
        return shopt_print(core, args, args.len() < 2);
    }

    let res = match args[1].as_str() {
        "-s" => core.shopts.set(&args[2], true),
        "-u" => core.shopts.set(&args[2], false),
        arg  => {
            eprintln!("sush: shopt: {}: invalid shell option name", arg);
            eprintln!("shopt: usage: shopt [-su] [optname ...]");
            false
        },
    };

    match res {
        true  => 0,
        false => 1,
    }
}
