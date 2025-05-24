//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error, ShellCore};
use crate::error::exec::ExecError;
use crate::utils::arg;
use super::parameter;

fn set_option(core: &mut ShellCore, opt: char, pm: char) {
    if pm == '+' {
        core.db.flags.retain(|e| e != opt);
        if opt == 'm' {
            let _ = core.options.set("monitor", false);
        }
    }else{
        if ! core.db.flags.contains(opt) {
            core.db.flags.push(opt);
        }
    }
}

pub fn set_options(core: &mut ShellCore, args: &[String]) -> Result<(), ExecError> {
    for a in args {
        if a.len() != 2 {
            return Err(ExecError::InvalidOption("set: ".to_owned() + &a.to_string()));
        }

        let pm = a.chars().nth(0).unwrap();
        let ch = a.chars().nth(1).unwrap();

        if (pm != '-' && pm != '+')
        || (pm == '+' && ch == 'r')
        || "rxveBH".find(ch).is_none() {
            return Err(ExecError::InvalidOption("set: ".to_owned() + &a.to_string()));
        }

        set_option(core, ch, pm);
    }
    Ok(())
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);

    if core.db.flags.contains('r') {
        if arg::consume_option("+r", &mut args) {
            let _ = super::error_exit(1, &args[0], "+r: invalid option", core);
            eprintln!("set: usage: set [-abefhkmnptuvxBCEHPT] [-o option-name] [--] [-] [arg ...]"); // TODO: this line is a dummy for test. We must implement all behaviors of these options.
            return 1;
        }
    }

    if args.len() <= 1 {
        return parameter::print_all(core);
    }

    if arg::consume_option("-m", &mut args) {
        if ! core.db.flags.contains('m') {
            core.db.flags += "m";
        }
        let _ = core.options.set("monitor", true);
        if args.len() <= 1 {
            return 0;
        }
    }

    if arg::consume_option("+m", &mut args) {
        core.db.flags.retain(|f| f != 'm');
        let _ = core.options.set("monitor", false);
        if args.len() <= 1 {
            return 0;
        }
    }

    if args[1].starts_with("--") {
        args[1] = core.db.position_parameters[0][0].clone();
        args.remove(0);
        return match parameter::set_positions(core, &args) {
            Ok(()) => 0,
            Err(e) => {
                return super::error_exit(1, &args[0], &String::from(&e), core);
            },
        }
    }

    if args[1] == "-o" || args[1] == "+o" {
        let positive = args[1] == "-o";

        if args.len() == 2 {
            core.options.print_all(positive);
            return 0;
        }else{
            if args[2] == "monitor" {
                if positive && ! core.db.flags.contains('m') {
                    core.db.flags.push('m');
                }else if ! positive {
                    core.db.flags.retain(|f| f != 'm');
                }
            }

            return match core.options.set(&args[2], positive) {
                Ok(())  => 0,
                Err(e) => {
                    return super::error_exit(2, &args[0], &String::from(&e), core);
                },
            };
        }
    }

    match args[1].starts_with("-") || args[1].starts_with("+") {
        true  => if let Err(e) = set_options(core, &args[1..]) {
            e.print(core);
            return 2;
        },
        false => if let Err(e) = parameter::set_positions(core, &args) {
            e.print(core);
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
        opt  => res = core.shopts.print_opt(opt, false),
    }

    match res {
        true  => 0,
        false => 1,
    }
}

pub fn shopt(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);
    let print = arg::consume_option("-p", &mut args);
    let o_opt = arg::consume_option("-o", &mut args);
    let q_opt = arg::consume_option("-q", &mut args);

    /* print section */
    if print && o_opt {
        if args.len() >= 2 && ! q_opt {
            core.options.print_opt(&args[1], true);
        }else if ! q_opt {
            core.options.print_all(false);
        }
        return 0;
    }

    if args.len() < 3 { // "shopt" or "shopt option"
        if ! q_opt {
            let len = args.len();
            return shopt_print(core, &mut args, len < 2);
        }
        return 0;
    }
    /* end of print section */

    if o_opt {
        let opt = match args[1].as_str() {
            "-s" => "-o",
            "-u" => "+o",
            other => other,
        }.to_string();
        let mut args_for_set = vec!["set".to_string(), opt];
        args_for_set.append(&mut args[2..].to_vec());

        return set(core, &mut args_for_set);
    }

    match args[1].as_str() { //TODO: args[3..] must to be set
        "-s" => {
            if core.shopts.implemented.contains(&args[2]) {
                match core.shopts.set(&args[2], true) {
                    Ok(()) => return 0,
                    Err(e) => {
                        e.print(core);
                        return 1;
                    },
                }
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
        "-u" => match core.shopts.set(&args[2], false) {
            Ok(()) => return 0,
            Err(e) => {
                e.print(core);
                return 1;
            },
        },
        arg  => {
            eprintln!("sush: shopt: {}: invalid shell option name", arg);
            eprintln!("shopt: usage: shopt [-su] [optname ...]");
            return 1;
        },
    }
}
