//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::exit;
use crate::core::data::Value;

fn print_data(k: &str, core: &mut ShellCore) {
    match core.data.get_value(k) {
        Some(Value::EvaluatedSingle(s)) => {
            println!("{}={}", k.to_string(), s.to_string()); 
        },
        Some(Value::EvaluatedArray(a)) => {
            let mut formatted = String::new();
            formatted += "(";
            for (i, v) in a.iter().enumerate() {
                formatted += &format!("[{}]=\"{}\" ", i, v).clone();
            }
            if formatted.ends_with(" ") {
                formatted.pop();
            }
            formatted += ")";
            println!("{}={}", k.to_string(), formatted); 
        },
        _ => {},
    }
}

fn print(core: &mut ShellCore) -> i32 {
    core.data.get_keys()
        .into_iter()
        .for_each(|k| print_data(&k, core));
    0
}

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

fn set_options(core: &mut ShellCore, args: &[String]) -> i32 {
    for a in args {
        if a.starts_with("--") {
            return 0;
        }
        let pm = a.chars().nth(0).unwrap();
        for ch in a[1..].chars() {
            if "xve".find(ch).is_none() {
                eprintln!("sush: set: {}{}: invalid option", &pm, &ch);
                return 2;
            }
            set_option(core, ch, pm);
        }
    }
    0
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match args.len() {
        0 => panic!("never come here"),
        1 => {
            match args[0] == "set" {
                true  => print(core),
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
