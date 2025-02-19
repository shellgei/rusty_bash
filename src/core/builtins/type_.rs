//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::{file_check, ShellCore};
use crate::utils::{arg, file};

fn type_no_opt(core: &mut ShellCore, com: &String) -> i32 {
    if core.aliases.contains_key(com) {
        println!("{} is aliased to `{}'", &com, &core.aliases[com]);
        return 0;
    }
    if core.db.functions.contains_key(com) {
        println!("{} is a function", &com);
        println!("{}", &core.db.functions[com].text);
        return 0;
    }
    if core.builtins.contains_key(com) {
        println!("{} is a shell builtin", com);
        return 0;
    }
    if let Some(path) = file::search_command(com) {
        println!("{} is {}", com, &path);
        return 0;
    }
    if file_check::is_executable(com) {
        println!("{} is {}", com, com);
        return 0;
    }
    1
}

fn type_p(core: &mut ShellCore, com: &String) -> i32 {
    if core.aliases.contains_key(com) 
    || core.db.functions.contains_key(com)
    || core.builtins.contains_key(com) {
        return 0;
    }

    if let Some(path) = file::search_command(com) {
        println!("{}", &path);
        return 0;
    }
    if file_check::is_executable(com) {
        println!("{}", com);
        return 0;
    }
    1
}

pub fn type_(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        return 0;
    }

    let mut exit_status = 0;
    let mut args = arg::dissolve_options(args);
    if arg::consume_option("-p", &mut args) {
        for a in &args[1..] {
             exit_status += type_p(core, a);
        }
        if exit_status > 1 {
            exit_status = 1;
        }
        return exit_status;
    }
    if arg::consume_option("-P", &mut args) {
        return 0;
    }

    for a in &args[1..] {
         exit_status += type_no_opt(core, a);
    }

    if exit_status > 1 {
        exit_status = 1;
    }
    exit_status
}
