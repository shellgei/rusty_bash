//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::builtins;
use crate::elements::substitution::Substitution;
use crate::utils::arg;
use crate::ShellCore;

pub(super) fn declare_p(core: &mut ShellCore, names: &[String], com: &str) -> i32 {
    for n in names {
        if ! core.db.exist(n) && ! core.db.exist_nameref(n) {
            return builtins::error_exit_text(1, n, "not found", core);
        }

        let mut opts: Vec<char> = core.db.get_flags(n).chars().collect();
        opts.sort();

        let mut opt: String = opts.into_iter().collect();
        while opt.len() == 0 {
            opt.push('-');
        }

        let prefix = match core.options.query("posix") {
            false => format!("declare -{opt} "),
            true => format!("{com} -{opt} "),
        };
        print!("{prefix}");
        core.db.print_for_declare(n);
    }
    0
}

pub(super) fn declare_print(core: &mut ShellCore, args: &[String]) -> i32 {

    if args.len() == 2 && args[1] == "-f" {
        let mut names: Vec<String> = core.db.functions.keys().map(|k| k.to_string()).collect();
        names.sort();

        for n in names {
            core.db.functions.get_mut(&n).unwrap().pretty_print(0);
        }
        return 0;
    }

    let mut names = core.db.get_param_keys();
    let mut options = String::new();

    if arg::has_option("-i", args) {
        names.retain(|n| core.db.has_flag(n, 'i'));
        options += "i";
    }

    if arg::has_option("-a", args) {
        names.retain(|n| core.db.is_array(n));
        options += "a";
    }

    if arg::has_option("-A", args) {
        names.retain(|n| core.db.is_assoc(n));
        options += "A";
    }

    if arg::has_option("-r", args) {
        names.retain(|n| core.db.is_readonly(n));
        if !core.options.query("posix") {
            options += "r";
        }
    }

    let prefix = format!("declare -{options}");
    for name in names {
        print!("{prefix}");
        if core.db.has_flag(&name, 'i') && !options.contains('i') {
            print!("i");
        }
        if core.db.has_flag(&name, 'x') && !options.contains('x') {
            print!("x");
        }
        if core.db.is_readonly(&name) && !options.contains('r') && !core.options.query("posix") {
            print!("r");
        }
        print!(" ");
        core.db.print_for_declare(&name);
    }
    0
}

pub(super) fn declare_print_function(core: &mut ShellCore, subs: &mut [Substitution]) -> i32 {
    let mut names: Vec<String> = subs.iter().map(|s| s.left_hand.name.clone()).collect();
    names.sort();

    if names.iter().all(|n| core.db.print_func(n)) {
        return 0;
    }
    1
}

pub(super) fn readonly_print(core: &mut ShellCore, args: &mut [String]) -> i32 {
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

    declare_p(core, &names, &args[0]);
    0
}
