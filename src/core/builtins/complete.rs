//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::{CompletionEntry, HashMap};
use crate::utils::arg;
use crate::{builtins, ShellCore};

fn action_to_reduce_symbol(arg: &str) -> String {
    match arg {
        "file" => "f",
        "directory" => "d",
        "command" => "c",
        "alias" => "a",
        "builtin" => "b",
        "group" => "g",
        "keyword" => "k",
        "variable" => "v",
        "export" => "e",
        "setopt" => "o",
        "job" => "j",
        "service" => "s",
        "user" => "u",
        _ => "",
    }
    .to_string()
}

fn opt_to_action(arg: &str) -> String {
    match arg {
        "-a" => "alias",
        "-b" => "builtin",
        "-c" => "command",
        "-d" => "directory",
        "-f" => "file",
        "-g" => "group",
        "-k" => "keyword",
        "-j" => "job",
        "-o" => "setopt",
        "-u" => "user",
        "-v" => "variable",
        _ => "",
    }
    .to_string()
}

fn print_complete(core: &mut ShellCore) -> i32 {
    if !core.completion.default_function.is_empty() {
        println!("complete -F {} -D", &core.completion.default_function);
    }

    for (name, info) in &core.completion.entries {
        if !info.function.is_empty() {
            print!("complete -F {} ", &info.function);
        } else if !info.action.is_empty() {
            let symbol = action_to_reduce_symbol(&info.action);

            if symbol.is_empty() {
                print!("complete -A {} ", &info.action);
            } else {
                print!("complete -{} ", &symbol);
            }

            if info.options.contains_key("-P") {
                print!("-P '{}' ", &info.options["-P"]);
            }
            if info.options.contains_key("-S") {
                print!("-S '{}' ", &info.options["-S"]);
            }
        } else {
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
        core.completion.default_function = args[1].clone();
        0
    } else {
        let func = args[1].clone();
        for command in &args[2..] {
            if !core.completion.entries.contains_key(command) {
                core.completion
                    .entries
                    .insert(command.clone(), CompletionEntry::default());
            }

            let info = &mut core.completion.entries.get_mut(command).unwrap();
            info.function = func.clone();
            info.o_options = o_options.clone();
        }

        0
    }
}

fn complete_r(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    for command in &mut args[1..] {
        core.completion.entries.remove(command);
    }

    0
}

pub fn complete(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 || args[1] == "-p" {
        return print_complete(core);
    }

    let mut o_options = vec![];
    let mut args = arg::dissolve_options(args);

    if arg::consume_option("-r", &mut args) {
        return complete_r(core, &mut args);
    }

    while let Some(v) = arg::consume_with_next_arg("-o", &mut args) {
        o_options.push(v);
    }

    let mut options = HashMap::new();
    let prefix = arg::consume_with_next_arg("-P", &mut args);
    if prefix.is_some() {
        options.insert("-P".to_string(), prefix.unwrap().clone());
    }
    let suffix = arg::consume_with_next_arg("-S", &mut args);
    if suffix.is_some() {
        options.insert("-S".to_string(), suffix.unwrap().clone());
    }

    let action = opt_to_action(&args[1]);
    if !action.is_empty() {
        for command in &args[2..] {
            if !core.completion.entries.contains_key(command) {
                core.completion
                    .entries
                    .insert(command.clone(), CompletionEntry::default());
            }

            let info = &mut core.completion.entries.get_mut(command).unwrap();
            info.action = action.clone();
            info.options = options.clone();
        }
        return 0;
    }

    if args.len() > 3 && args[1] == "-A" {
        for command in &args[3..] {
            if !core.completion.entries.contains_key(command) {
                core.completion
                    .entries
                    .insert(command.clone(), CompletionEntry::default());
            }

            let info = &mut core.completion.entries.get_mut(command).unwrap();
            info.action = args[2].clone();
            info.options = options.clone();
        }

        return 0;
    }

    if arg::consume_option("-F", &mut args) {
        complete_f(core, &mut args, &o_options)
    } else {
        let msg = format!("{}: still unsupported", &args[1]);
        builtins::error_exit(1, &args[0], &msg, core)
    }
}
