//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::{error, exit};
use super::parameter;

pub fn set_parameters(core: &mut ShellCore, args: &[String]) -> i32 {
    match core.data.position_parameters.pop() {
        None => exit::internal("empty param stack"),
        _    => {},
    }
    core.data.position_parameters.push(args.to_vec());
    core.data.set_param("#", &(args.len()-1).to_string());
    0
}

fn set_option(core: &mut ShellCore, opt: char, pm: char) {
    if pm == '+' {
        core.data.flags.retain(|e| e != opt);
    }else{
        if ! core.data.flags.contains(opt) {
            core.data.flags.push(opt);
        }
    }
}

pub fn set_options(core: &mut ShellCore, args: &[String]) -> i32 {
    for a in args {
        if a.starts_with("--") {
            return 0;
        }
        let pm = a.chars().nth(0).unwrap();
        for ch in a[1..].chars() {
            if "xveB".find(ch).is_none() {
                eprintln!("sush: set: {}{}: invalid option", &pm, &ch);
                return 2;
            }
            set_option(core, ch, pm);
        }
    }
    0
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    args[0] = core.data.get_param("0");
    match args.len() {
        0 => panic!("never come here"),
        1 => {
            match args[0] == "set" {
                true  => parameter::print_all(core),
                false => set_parameters(core, args),
            }
        },
        _ => {
            if args[1].starts_with("--") {
                args.remove(0);
                return set_parameters(core, args)
            }

            if args[1] == "-o" {
                if args.len() == 2 {
                    core.options.print_all();
                    return 0;
                }else{
                    match core.options.set(&args[2], true) {
                        true  => return 0,
                        false => return 2,
                    }
                }
            }

            if args[1] == "+o" {
                if args.len() == 2 {
                    core.options.print_all2();
                    return 0;
                }else{
                    match core.options.set(&args[2], false) {
                        true  => return 0,
                        false => return 2,
                    }
                }
            }

            match args[1].starts_with("-") || args[1].starts_with("+") {
                true  => set_options(core, &args[1..]),
                false => set_parameters(core, args),
            }
        },
    }
}

pub fn shift(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
        let mut last = core.data.position_parameters.pop().unwrap();
        if last.len() > 1 {
            last.remove(1);
        }
        core.data.position_parameters.push(last);
        return 0;
    }

    if args.len() == 2 {
        let n = match args[1].parse::<i32>() {
            Ok(n) => n,
            Err(_) => {
                let err = format!("shift: {}: numeric argument required", &args[1]);
                error::print(&err, core);
                return 1;
            },
        };

        if n < 0 {
            let err = format!("shift: {}: shift count out of range", &args[1]);
            error::print(&err, core);
            return 1;
        }

        let mut last = core.data.position_parameters.pop().unwrap();
        for _ in 0..n {
            if last.len() == 1 {
                break;
            }
            last.remove(1);
        }
        core.data.position_parameters.push(last);
        return 0;
    }

    error::print("shift: too many arguments", core);
    1
}

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
