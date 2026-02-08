//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use crate::utils::arg;
use crate::{error, ShellCore};

pub fn set_positions(core: &mut ShellCore, args: &[String]) -> Result<(), ExecError> {
    let com = match core.db.position_parameters.pop() {
        Some(scope) => {
            if scope.is_empty() {
                "".to_string()
            } else {
                scope[0].clone()
            }
        }
        None => return Err(ExecError::Other("empty param stack".to_string())),
    };

    let mut tmp = args.to_vec();
    if !tmp.is_empty() {
        tmp[0] = com;
    } else {
        tmp.push(com);
    }

    core.db.position_parameters.push(tmp);
    Ok(())
}

pub fn set_positions_c(core: &mut ShellCore, args: &[String]) -> Result<(), ExecError> {
    if core.db.position_parameters.pop().is_none() {
        return Err(ExecError::Other("empty param stack".to_string()));
    }

    core.db.position_parameters.push(args.to_vec());
    Ok(())
}

fn check_invalid_options(args: &[String]) -> Result<(), ExecError> {
    for a in args {
        if a.starts_with("-") {
            return Err(ExecError::InvalidOption(
                "set: ".to_owned() + &a.to_string(),
            ));
        }
    }
    Ok(())
}

pub fn set_options(core: &mut ShellCore, args: &mut Vec<String>) -> Result<(), ExecError> {
    set_short_options(core, args);
    check_invalid_options(args)
}

pub fn set_short_options(core: &mut ShellCore, args: &mut Vec<String>) {
    for (short, long) in [
        ('a', "allexport"),
        ('t', "onecmd"),
        ('m', "monitor"),
        ('C', "noclobber"),
        ('a', "allexport"),
        ('B', "braceexpand"),
        ('u', ""),
        ('e', ""),
        ('r', ""),
        ('H', ""),
        ('x', ""),
        ('v', ""),
    ] {
        let minus_opt = format!("-{short}");
        let plus_opt = format!("+{short}");

        if arg::consume_option(&minus_opt, args) {
            if !core.db.flags.contains(short) {
                core.db.flags += &minus_opt[1..];
            }
            if !long.is_empty() {
                let _ = core.options.set(long, true);
            }
        }

        if arg::consume_option(&plus_opt, args) {
            core.db.flags.retain(|f| f != short);
            if !long.is_empty() {
                let _ = core.options.set(long, false);
            }
        }
    }
}

pub fn set(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = arg::dissolve_options(args);

    if core.db.flags.contains('r') && arg::consume_arg("+r", &mut args) {
        let _ = super::error_(1, &args[0], "+r: invalid option", core);
        eprintln!("set: usage: set [-abefhkmnptuvxBCEHPT] [-o option-name] [--] [-] [arg ...]");
        return 1;
    }

    if args.len() <= 1 {
        core.db.print_params_and_funcs();
        return 0;
    }

    set_short_options(core, &mut args);

    if args.len() < 2 {
        return 0;
    }

    if args[1] == "--" || args[1] == "-" {
        args[1] = core.db.position_parameters[0][0].clone();
        args.remove(0);
        match set_positions(core, &args) {
            Ok(()) => return 0,
            Err(e) => {
                return super::error_(1, &args[0], &String::from(&e), core);
            }
        }
    }

    if args[1] == "-o" || args[1] == "+o" {
        let positive = args[1] == "-o";

        if args.len() == 2 {
            core.options.print_all(positive);
            return 0;
        } else {
            if args[2] == "monitor" {
                if positive && !core.db.flags.contains('m') {
                    core.db.flags.push('m');
                } else if !positive {
                    core.db.flags.retain(|f| f != 'm');
                }
            }

            return match core.options.set(&args[2], positive) {
                Ok(()) => 0,
                Err(e) => {
                    return super::error_(2, &args[0], &String::from(&e), core);
                }
            };
        }
    }

    if !args[1].starts_with("-") && !args[1].starts_with("+") {
        if let Err(e) = set_positions(core, &args) {
            e.print(core);
            return 2;
        } else {
            return 0;
        }
    }

    if let Err(e) = check_invalid_options(&args) {
        e.print(core);
        return 2;
    }
    0
}

pub fn shift(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
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
            }
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

pub fn shopt_print(core: &mut ShellCore, args: &[String], all: bool) -> i32 {
    if all {
        core.shopts.print_all(true);
        return 0;
    }

    let mut res = true;
    match args[1].as_str() {
        "-s" => core.shopts.print_if(true),
        "-u" => core.shopts.print_if(false),
        "-q" => return 0,
        opt => res = core.shopts.print_opt(opt, false),
    }

    match res {
        true => 0,
        false => 1,
    }
}

pub fn shopt(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = arg::dissolve_options(args);
    let print = arg::consume_arg("-p", &mut args);
    let o_opt = arg::consume_arg("-o", &mut args);
    let q_opt = arg::consume_arg("-q", &mut args);

    /* print section */
    if print && o_opt {
        if args.len() >= 2 && !q_opt {
            core.options.print_opt(&args[1], true);
        } else if !q_opt {
            core.options.print_all(false);
        }
        return 0;
    }

    /* q option */
    if q_opt {
        for a in &args[1..] {
            if ! core.shopts.query(a) {
                return 1;
            }
        }
    }

    if args.len() < 3 {
        // "shopt" or "shopt option"
        if !q_opt {
            let len = args.len();
            return shopt_print(core, &args, len < 2);
        }
        return 0;
    }
    /* end of print section */

    if o_opt {
        let opt = match args[1].as_str() {
            "-s" => "-o",
            "-u" => "+o",
            other => other,
        }
        .to_string();
        let mut args_for_set = vec!["set".to_string(), opt];
        args_for_set.append(&mut args[2..].to_vec());

        return set(core, &args_for_set);
    }

    match args[1].as_str() {
        //TODO: args[3..] must to be set
        "-s" => {
            if core.shopts.implemented.contains(&args[2]) {
                match core.shopts.set(&args[2], true) {
                    Ok(()) => 0,
                    Err(e) => {
                        e.print(core);
                        1
                    }
                }
            } else {
                let msg = format!("shopt: {}: not supported yet", &args[2]);
                error::print(&msg, core);
                1
            }
        }
        "-q" => {
            for arg in &args[2..] {
                if !core.shopts.exist(arg) {
                    let msg = format!("shopt: {}: invalid shell option name", &arg);
                    error::print(&msg, core);
                    return 1;
                }
                if !core.shopts.query(arg) {
                    return 1;
                }
            }
            0
        }
        "-u" => match core.shopts.set(&args[2], false) {
            Ok(()) => 0,
            Err(e) => {
                e.print(core);
                1
            }
        },
        arg => {
            eprintln!("sush: shopt: {arg}: invalid shell option name");
            eprintln!("shopt: usage: shopt [-su] [optname ...]");
            1
        }
    }
}
