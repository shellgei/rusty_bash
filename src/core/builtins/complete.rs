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
        "export" => "e",
        "group" => "g",
        "keyword" => "k",
        "variable" => "v",
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
        "-e" => "export",
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

fn print_each_complete(name: &str, info: &CompletionEntry) -> i32 {
    let mut o_options = String::new();

    for oopt in &info.o_options {
        o_options.push_str(&format!("-o {} ", &oopt));
    }

    if !info.large_w_cands.is_empty() {
        if info.large_w_cands.starts_with('"') {
            print!("complete {}-W '{}' ", &o_options, &info.large_w_cands);
        }else{
            print!("complete {}-W {} ", &o_options, &info.large_w_cands);
        }
    } else if !info.function.is_empty() {
        print!("complete {}-F {} ", &o_options, &info.function);
    } else if !info.action.is_empty() {
        let symbol = action_to_reduce_symbol(&info.action);

        if symbol.is_empty() {
            print!("complete {}-A {} ", &o_options, &info.action);
        } else {
            print!("complete {}-{} ", &o_options, &symbol);
        }

        for opt in ["-X", "-G", "-W", "-P", "-S"] {
            if info.options.contains_key(opt) {
                print!("{} '{}' ", opt, &info.options[opt]);
            }
        }
    } else {
        print!("complete {}", &o_options);
    }
    println!("{}", &name);
    0
}

fn print_complete_all(core: &mut ShellCore) -> i32 {
    if !core.completion.default_function.is_empty() {
        println!("complete -F {} -D", &core.completion.default_function);
    }


    for (name, info) in &mut core.completion.entries {
        print_each_complete(&name, info);
    }
    0
}

fn print_complete(coms: &[String], core: &mut ShellCore) -> i32 {
    let mut err = false;
    for name in coms {
        if let Some(info) = core.completion.entries.get_mut(name) {
            print_each_complete(&name, info);
        }else{
            let err_str = format!("{}: no completion specification", &name);
            err = 0 != builtins::error_(1, "complete", &err_str, core);
        };
    }
    if err {
        return 1;
    }
    0
}

fn complete_f(core: &mut ShellCore, args: &[String], o_options: &[String]) -> i32 {
    let d_option = arg::has_option("-D", args);
    let mut arg_index = 1;
    if d_option {
        arg_index = 2;
    }

    if args.len() <= arg_index {
        return builtins::error_(2, &args[0], "-F: option requires an argument", core);
    }

    if d_option {
        core.completion.default_function = args[arg_index].clone();
        0
    } else {
        let func = args[arg_index].clone();
        for command in &args[arg_index + 1..] {
            if !core.completion.entries.contains_key(command) {
                core.completion
                    .entries
                    .insert(command.clone(), CompletionEntry::default());
            }

            let info = &mut core.completion.entries.get_mut(command).unwrap();
            info.function = func.clone();
            info.o_options = o_options.to_owned();
        }

        0
    }
}

fn complete_large_w(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = args.to_vec();
    let com = args.pop().unwrap();

    core.completion
        .entries
        .insert(com.clone(), CompletionEntry::default());

    let info = &mut core.completion.entries.get_mut(&com).unwrap();
    info.large_w_cands = args[2].clone();
    0
}

fn complete_r(core: &mut ShellCore, args: &[String]) -> i32 {
    for command in &args[1..] {
        core.completion.entries.remove(command);
    }

    0
}

pub fn complete(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    if args.len() <= 1 {
        return print_complete_all(core);
    }else if args[1] == "-p" && args.len() == 2 {
        return print_complete_all(core);
    }else if args[1] == "-p" {
        return print_complete(&args[2..], core);
    }

    let mut o_options = vec![];
    let mut args = arg::dissolve_options(&args);

    if args[1] == "-W" {
        return complete_large_w(core, &args);
    }

    if arg::consume_arg("-r", &mut args) {
        return complete_r(core, &args);
    }

    while let Some(v) = arg::consume_with_next_arg("-o", &mut args) {
        o_options.push(v);
    }
    o_options.sort();

    let mut options = HashMap::new();
    for opt in ["-X", "-G", "-W", "-P", "-S"] {
        let prefix = arg::consume_with_next_arg(opt, &mut args);
        if let Some(prefix) = prefix {
            options.insert(opt.to_string(), prefix.clone());
        }
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
            info.o_options = o_options.clone();
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

    if arg::consume_arg("-F", &mut args) {
        complete_f(core, &args, &o_options)
    } else {
        let msg = format!("{}: still unsupported", &args[1]);
        builtins::error_(1, &args[0], &msg, core)
    }
}
