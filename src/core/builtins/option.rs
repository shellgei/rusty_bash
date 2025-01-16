//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error, ShellCore};
use crate::error::exec;
use crate::error::exec::ExecError;
use crate::utils::arg;
use super::parameter;

fn set_option(core: &mut ShellCore, opt: char, pm: char) {
    if pm == '+' {
        core.db.flags.retain(|e| e != opt);
    }else{
        if ! core.db.flags.contains(opt) {
            core.db.flags.push(opt);
        }
    }
}

pub fn set_options(core: &mut ShellCore, args: &[String]) -> Result<(), ExecError> {
    for a in args {
        if a.len() != 2 {
            return Err(ExecError::InvalidOption(a.to_string()));
            /*
            error::internal("invalid option");
            return 1;
            */
        }

        let pm = a.chars().nth(0).unwrap();
        let ch = a.chars().nth(1).unwrap();

        if pm != '-' && pm != '+' {
            return Err(ExecError::InvalidOption(a.to_string()));
            /*
            error::internal("not an option");
            return 1;
            */
        }else if "xveB".find(ch).is_none() {
            return Err(ExecError::InvalidOption(a.to_string()));
            /*
            eprintln!("sush: set: {}: invalid option", &a);
            return 2;
            */
        }

        set_option(core, ch, pm);
    }
    Ok(())
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);

    if args.is_empty() {
        panic!("never come here");
    }else if args.len() == 1 {
        return parameter::print_all(core);
    }

    if args[1].starts_with("--") {
        args.remove(0);
        return match parameter::set_positions(core, &args) {
            Ok(()) => 0,
            Err(e) => {
                exec::print_error(e, core);
                return 1;
            },
        }
    }

    if args[1] == "-o" || args[1] == "+o" {
        let positive = args[1] == "-o";

        if args.len() == 2 {
            core.options.print_all(positive);
            return 0;
        }else{
            if args[2] == "noglob" {
                eprintln!("{}: not supprted yet", &args[2]);
                return 1;
            }
            return match core.options.set(&args[2], positive) {
                true  => 0,
                false => 2,
            };
        }
    }

    match args[1].starts_with("-") || args[1].starts_with("+") {
        true  => if let Err(e) = set_options(core, &args[1..]) {
            exec::print_error(e, core);
            return 2;
        },
        false => if let Err(e) = parameter::set_positions(core, &args) {
            exec::print_error(e, core);
            return 2;
        },
    }
    0
}

pub fn shift(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
        let mut last = core.db.position_parameters.pop().unwrap();
        if last.len() > 1 {
            last.remove(1);
        }
        core.db.position_parameters.push(last);
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

        let mut last = core.db.position_parameters.pop().unwrap();
        for _ in 0..n {
            if last.len() == 1 {
                break;
            }
            last.remove(1);
        }
        core.db.position_parameters.push(last);
        return 0;
    }

    error::print("shift: too many arguments", core);
    1
}

pub fn shopt_print(core: &mut ShellCore, args: &mut Vec<String>, all: bool) -> i32 {
    if all {
        core.shopts.print_all(true);
        return 0;
    }

    let mut res = true;
    match args[1].as_str() {
        "-s" => core.shopts.print_if(true),
        "-u" => core.shopts.print_if(false),
        "-q" => return 0,
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
        "-s" => {
            if ["extglob", "progcomp"].iter().any(|&e| e == args[2]) {
                core.shopts.set(&args[2], true)
            }else{
                let msg = format!("shopt: {}: not supported yet", &args[2]);
                error::print(&msg, core);
                return 1;
            }
        },
        "-q" => {
            for arg in &args[2..] {
                if ! core.shopts.exist(arg) {
                    let msg = format!("shopt: {}: invalid shell option name", &arg);
                    error::print(&msg, core);
                    return 1;
                }
                if ! core.shopts.query(arg) {
                    return 1;
                }
            }
            return 0;
        },
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
