//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{builtins, ShellCore};
use crate::core::{CompletionInfo, HashMap};
use crate::utils::arg;

fn action_to_reduce_symbol(arg: &str) -> String {
    match arg {
        "file" => "f",
        "directory" => "d",
        "command" => "c",
        "alias" => "a",
        "builtin" => "b",
        "keyword" => "k",
        "variable" => "v",
        "export" => "e",
        "setopt" => "o",
        "job" => "j",
        "service" => "s",
        "user" => "u",
        "group" => "g",
        _ => "",
    }.to_string()
}

fn opt_to_action(arg: &str) -> String {
    match arg {
        "-a" => "alias",
        "-b" => "builtin",
        "-c" => "command",
        "-j" => "job",
        "-o" => "setopt",
        "-u" => "user",
        "-v" => "variable",
        _ => "",
    }.to_string()
}

fn print_complete(core: &mut ShellCore) -> i32 {
    if core.default_completion_functions != "" {
        println!("complete -F {} -D", &core.default_completion_functions);
    }

    for (name, info) in &core.completion_info {
        if info.function != "" {
            print!("complete -F {} ", &info.function);
        }else if info.action != "" {
            let symbol = action_to_reduce_symbol(&info.action);

            if symbol == "" {
                print!("complete -A {} ", &info.action);
            }else{
                print!("complete -{} ", &symbol);
            }

            if info.options.contains_key("-P") {
                print!("-P '{}' ", &info.options["-P"]);
            }
            if info.options.contains_key("-S") {
                print!("-S '{}' ", &info.options["-S"]);
            }
        }else{
            print!("complete ");
        }
        println!("{}", &name); 
    }
    0
}

fn complete_f(core: &mut ShellCore, args: &mut Vec<String>, o_options: &Vec<String>) -> i32 {
    let d_option = arg::consume_option("-D", args);

    if args.len() <= 1 {
        return builtins::error_exit(2, &args[0], "-F: option requires an argument", core);
    }
 
    if d_option {
        core.default_completion_functions = args[1].clone();
        return 0;
    }else {
        let func = args[1].clone();
        for command in &args[2..] {
            if ! core.completion_info.contains_key(command) {
                core.completion_info.insert(command.clone(), CompletionInfo::default());
            }
    
            let info = &mut core.completion_info.get_mut(command).unwrap();
            info.function = func.clone();
            info.o_options = o_options.clone();
        }

        return 0;
    }
}

pub fn complete(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 || args[1] == "-p" {
        return print_complete(core);
    }

    let mut o_options = vec![];
    let mut args = arg::dissolve_options(args);
    while let Some(v) = arg::consume_with_next_arg("-o", &mut args) {
        o_options.push(v);
    }

    let mut options = HashMap::new();
    let prefix = arg::consume_with_next_arg("-P", &mut args);
    if prefix != None {
        options.insert("-P".to_string(), prefix.unwrap().clone());
    }
    let suffix = arg::consume_with_next_arg("-S", &mut args);
    if suffix != None {
        options.insert("-S".to_string(), suffix.unwrap().clone());
    }

    let action = opt_to_action(&args[1]);
    if action != "" {
        for command in &args[2..] {
            if ! core.completion_info.contains_key(command) {
                core.completion_info.insert(command.clone(), CompletionInfo::default());
            }
    
            let info = &mut core.completion_info.get_mut(command).unwrap();
            info.action = action.clone();
            info.options = options.clone();
        }
        return 0;
    }

    if args.len() > 3 && args[1] == "-A" {
        for command in &args[3..] {
            if ! core.completion_info.contains_key(command) {
                core.completion_info.insert(command.clone(), CompletionInfo::default());
            }
    
            let info = &mut core.completion_info.get_mut(command).unwrap();
            info.action = args[2].clone();
            info.options = options.clone();
        }

        return 0;
    }

    if arg::consume_option("-F", &mut args) {
        complete_f(core, &mut args, &o_options)
    }else{
        let msg = format!("{}: still unsupported", &args[1]);
        builtins::error_exit(1, &args[0], &msg, core)
    }
}

/*
fn compopt_set(info: &mut CompletionInfo, plus: &Vec<String>, minus: &Vec<String>) -> i32 {
    for opt in minus { //add
        if ! info.o_options.contains(opt) {
            info.o_options.push(opt.to_string());
        }
    }

    for opt in plus { //remove
        info.o_options.retain(|e| e != opt);
    }

    0
}

fn compopt_print(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let optlist = vec!["bashdefault", "default",
                       "dirnames", "filenames", "noquote",
                       "nosort", "nospace", "plusdirs"];
    let optlist: Vec<String> = optlist.iter().map(|s| s.to_string()).collect();

    let com = args[1].clone();
    if core.completion_info.contains_key(&com) {
        let info = &core.completion_info.get_mut(&com).unwrap();

        print!("compopt ");
        for opt in &optlist {
            match info.o_options.contains(opt) {
                true  => print!("-o {} ", opt), 
                false => print!("+o {} ", opt), 
            }
        }
        println!("{}", &com);
    }else{
        eprintln!("sush: compopt: {}: no completion specification", &args[1]);
        return 1;
    }

    0
}

pub fn compopt(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        dbg!("{:?}", &core.completion_info);
        return 1;
    }

    if ! args[1].starts_with("-") && ! args[1].starts_with("+") {
        return compopt_print(core, args);
    }

    let mut flag = "".to_string();
    let mut minus = vec![];
    let mut plus = vec![];
    let mut minus_d = vec![];
    let mut plus_d = vec![];
    let mut minus_e = vec![];
    let mut plus_e = vec![];

    while args.len() > 1 {
        if args[1] == "-D" || args[1] == "-E" {
            flag = args[1].clone();
            args.remove(1);
            continue;
        }

        if args[1] == "-o" {
            let opt = arg::consume_with_next_arg("-o", args);
            if opt.is_none() {
                return 1;
            }

            match flag.as_str() { 
                ""   => minus.push(opt.unwrap()),
                "-D" => minus_d.push(opt.unwrap()),
                "-E" => minus_e.push(opt.unwrap()),
                _ => return 1,
            }
            continue;
        }

        if args[1] == "+o" {
            let opt = arg::consume_with_next_arg("+o", args);
            if opt.is_none() {
                return 1;
            }

            match flag.as_str() { 
                ""   => plus.push(opt.unwrap()),
                "-D" => plus_d.push(opt.unwrap()),
                "-E" => plus_e.push(opt.unwrap()),
                _ => return 1,
            }
            continue;
        }

        break;
    }

    let info = if args.len() == 1 {
        &mut core.current_completion_info
    }else if args.len() == 2 {
        match core.completion_info.get_mut(&args[1]) {
            Some(i) => i,
            None => return 1,
        }
    }else{
        return 1;
    };
    return compopt_set(info, &plus, &minus);

    //TODO: support of -D -E
}
*/
