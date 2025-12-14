//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::builtins;
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

pub(super) fn p_option(core: &mut ShellCore, names: &[String], com: &str) -> i32 {
    for n in names {
        if ! core.db.exist(n) && ! core.db.exist_nameref(n) {
            return builtins::error_exit_text(1, n, "not found", core);
        }

        let opt = format_options(n, core);
        match core.options.query("posix") {
            false => print!("declare {opt} "),
            true => print!("{com} {opt} "),
        };
        core.db.print_for_declare(n);
    }
    0
}

fn select_with_flags(core: &mut ShellCore,
                     names: &mut Vec<String>, args: &[String]) {
    for flag in ['i', 'a', 'A', 'r'] {
        let opt = "-".to_owned() + &flag.to_string();
        if arg::has_option(&opt, args) {
            names.retain(|n| core.db.has_flag(n, flag));
        }
    }
    /*
    if arg::has_option("-i", args) {
        names.retain(|n| core.db.has_flag(n, 'i'));
    }

    if arg::has_option("-a", args) {
        names.retain(|n| core.db.is_array(n));
    }

    if arg::has_option("-A", args) {
        names.retain(|n| core.db.is_assoc(n));
    }

    if arg::has_option("-r", args) {
        names.retain(|n| core.db.is_readonly(n));
    }*/
}

pub(super) fn declare_print(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() == 2 && args[1] == "-f" {
        let mut names: Vec<String> = core.db.functions.keys().map(|k| k.to_string()).collect();
        names.sort();

        if names.iter().all(|n| core.db.print_func(n)) {
            return 0;
        }
        return 1;
    }

    let mut names = core.db.get_param_keys();
    select_with_flags(core, &mut names, args);

    for name in names {
        let mut options = format_options(&name, core);
        if core.options.query("posix") {
            options.retain(|e| e != 'r');
        }

        print!("declare {options} ");
        core.db.print_for_declare(&name);
    }
    0
}

pub(super) fn functions(core: &mut ShellCore, subs: &mut [Substitution]) -> i32 {
    let mut names: Vec<String> = subs.iter().map(|s| s.left_hand.name.clone()).collect();
    names.sort();

    if names.iter().all(|n| core.db.print_func(n)) {
        return 0;
    }
    1
}

pub(super) fn readonly_params(core: &mut ShellCore, args: &mut [String]) -> i32 {
    let array_opt = arg::has_option("-a", args);
    let assoc_opt = arg::has_option("-A", args);
    let int_opt = arg::has_option("-i", args);

    let mut names: Vec<String> = core
        .db
        .get_param_keys()
        .iter()
        .filter(|e| core.db.is_readonly(e))
        .map(|e| e.to_string())
        .collect();

    if array_opt {
        names.retain(|e| core.db.is_array(e));
    }
    if assoc_opt {
        names.retain(|e| core.db.is_assoc(e));
    }
    if int_opt {
        names.retain(|e| core.db.is_single_num(e));
    }

    p_option(core, &names, &args[0]);
    0
}
