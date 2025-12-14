//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::builtins;
use crate::core::DataBase;
use crate::elements::substitution::Substitution;
use crate::utils::arg;
use crate::ShellCore;

fn format_options(name: &String, core: &mut ShellCore) -> String {
    let mut opts: Vec<char> = core.db.get_flags(name).chars().collect();
    opts.sort();

    let ans: String = opts.into_iter().collect();
    match ans.len() {
        0 => "--".to_string(),
        _ =>  "-".to_owned() + &ans,
    }
}

pub(super) fn p_option(core: &mut ShellCore, names: &[String], args: &[String]) -> i32 {
    let mut names = names.to_vec();
    select_with_flags(core, &mut names, args);

    for n in &names {
        if ! core.db.exist(n) && ! core.db.exist_nameref(n) {
            return builtins::error_exit_text(1, n, "not found", core);
        }

        let opt = format_options(n, core);
        match core.options.query("posix") {
            false => print!("declare {opt} "),
            true => print!("{} {} ", &args[0], opt),
        };
        core.db.print_for_declare(n);
    }
    0
}

fn select_with_flags(core: &mut ShellCore, names: &mut Vec<String>, args: &[String]) {
    for flag in ['i', 'a', 'A', 'r', 'x'] {
        let opt = "-".to_owned() + &flag.to_string();
        if arg::has_option(&opt, args) {
            names.retain(|n| core.db.has_flag(n, flag));
        }
    }
}

pub(super) fn match_args_params(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut names = core.db.get_param_keys();
    select_with_flags(core, &mut names, args);

    for name in names {
        let mut options = format_options(&name, core);
        if core.options.query("posix") {
            options.retain(|e| e != 'r');
        }

        match core.options.query("posix") {
            false => print!("declare {options} "),
            true => print!("{} {} ", &args[0], options),
        };
        //print!("declare {options} ");
        core.db.print_for_declare(&name);
    }
    0
}

pub(super) fn all_functions(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() != 2 {
        return 0;
    }

    let mut names = core.db.get_func_keys();
    names.sort();

    if names.iter().all(|n| core.db.print_func(n)) {
        return 0;
    }
    1
}

pub(super) fn functions(core: &mut ShellCore, args: &[String],
                        subs: &mut [Substitution]) -> i32 {
    if args.len() != 2 {
        return 0;
    }

    let mut names: Vec<String> = subs.
        iter()
        .map(|s| s.left_hand.name.clone())
        .collect();
    names.sort();

    if names.iter().all(|n| core.db.print_func(n)) {
        return 0;
    }
    1
}

pub(super) fn all_match(core: &mut ShellCore, args: &mut [String]) -> i32 {
    if args.len() <= 1 {
        DataBase::print_params_and_funcs(core);
        return 0;
    }
    if arg::has_option("-f", &args) {
        return all_functions(core, &args);
    }
    match_args_params(core, &args)
}
