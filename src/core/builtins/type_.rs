//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::{arg, file};
use crate::{file_check, utils, ShellCore};

fn type_no_opt_sub(core: &mut ShellCore, com: &String) -> i32 {
    //if core.aliases.contains_key(com) {
    if core.db.has_array_value("BASH_ALIASES", com) {
        let alias = core.db.get_elem("BASH_ALIASES", com).unwrap();
        println!("{} is aliased to `{}'", &com, &alias);
        return 0;
    }
    if utils::reserved(com) {
        println!("{} is a shell keyword", &com);
        return 0;
    }
    if core.db.functions.contains_key(com) {
        println!("{} is a function", &com);
        if let Some(val) = core.db.functions.get_mut(com) {
            val.pretty_print(0);
        };
        return 0;
    }
    if core.builtins.contains_key(com) {
        println!("{com} is a shell builtin");
        return 0;
    }
    if let Some(path) = file::search_command(com) {
        println!("{} is {}", com, &path);
        return 0;
    }
    if file_check::is_executable(com) {
        println!("{com} is {com}");
        return 0;
    }
    1
}

fn type_no_opt(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut exit_status = 0;
    for a in args {
        exit_status += type_no_opt_sub(core, a);
    }
    if exit_status > 1 {
        exit_status = 1;
    }
    exit_status
}

fn type_t(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut exit_status = 0;
    for a in args {
        exit_status += type_t_sub(core, a);
    }
    if exit_status > 1 {
        exit_status = 1;
    }
    exit_status
}

fn type_t_sub(core: &mut ShellCore, com: &String) -> i32 {
    if core.db.has_array_value("BASH_ALIASES", com) {
        println!("alias");
        return 0;
    }
    if utils::reserved(com) {
        println!("keyword");
        return 0;
    }
    if core.db.functions.contains_key(com) {
        println!("function");
        return 0;
    }
    if core.builtins.contains_key(com) {
        println!("builtin");
        return 0;
    }
    if file::search_command(com).is_some() || file_check::is_executable(com) {
        println!("file");
        return 0;
    }

    1
}

fn type_p(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut exit_status = 0;
    for a in args {
        exit_status += type_p_sub(core, a);
    }
    if exit_status > 1 {
        exit_status = 1;
    }
    exit_status
}

fn type_large_p(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut exit_status = 0;
    for a in args {
        exit_status += type_large_p_sub(core, a);
    }
    if exit_status > 1 {
        exit_status = 1;
    }
    exit_status
}

fn type_p_sub(core: &mut ShellCore, com: &String) -> i32 {
    if core.db.has_array_value("BASH_ALIASES", com)
        || core.db.functions.contains_key(com)
        || utils::reserved(com)
        || core.builtins.contains_key(com)
    {
        return 0;
    }

    if let Some(path) = file::search_command(com) {
        println!("{}", &path);
        return 0;
    }
    if file_check::is_executable(com) {
        println!("{com}");
        return 0;
    }
    1
}

fn type_large_p_sub(core: &mut ShellCore, com: &String) -> i32 {
    let mut es = 1;
    if core.db.has_array_value("BASH_ALIASES", com)
        || core.db.functions.contains_key(com)
        || utils::reserved(com)
        || core.builtins.contains_key(com)
    {
        es = 0;
    }

    if let Some(path) = file::search_command(com) {
        println!("{}", &path);
        return 0;
    }
    if file_check::is_executable(com) {
        println!("{com}");
        return 0;
    }
    es
}

pub fn type_(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() < 2 {
        return 0;
    }

    let mut args = arg::dissolve_options(args);

    let t_option = arg::consume_arg("-t", &mut args);
    if t_option {
        if args.len() > 1 && args[1] == "--" {
            args.remove(1);
        }
        return type_t(core, &args[1..]);
    }
    let p_option = arg::consume_arg("-p", &mut args);
    if p_option {
        if args[1] == "--" {
            args.remove(1);
        }
        return type_p(core, &args[1..]);
    }
    let large_p_option = arg::consume_arg("-P", &mut args);
    if large_p_option {
        if args[1] == "--" {
            args.remove(1);
        }
        return type_large_p(core, &args[1..]);
    }

    type_no_opt(core, &args[1..])
}
