//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::arg;
use crate::elements::substitution::Substitution;
use crate::error::exec::ExecError;

fn print_args_match(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() <= 1 { 
        core.db.print_params_and_funcs();
        return 0;
    }
    print_args_match_params(core, args)
}

fn print_params_match(core: &mut ShellCore,
                      subs: &mut [Substitution]) -> i32 {
    let names = subs.iter()
                    .map(|e| e.left_hand.text.clone())
                    .collect::<Vec<String>>();

    names.iter().for_each(|n| {print_with_flags(core, n); });
    0
}

fn print_args_match_params(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut names = core.db.get_param_keys();
    drop_by_args(core, &mut names, args);
    names.iter().for_each(|n| {print_with_flags(core, n); });
    0
}

fn drop_by_args(core: &mut ShellCore, names: &mut Vec<String>, args: &[String]) {
    for flag in ['i', 'a', 'A', 'r', 'x', 'u', 'n', 'l'] {
        let opt = "-".to_owned() + &flag.to_string();
        if arg::has_option(&opt, args) {
            names.retain(|n| core.db.has_flag(n, flag));
        }
    }
}

fn print_with_flags(core: &mut ShellCore, name: &String) {
    let mut flags = core.db.get_flags(name).to_string();
    if flags.is_empty() {
        flags.push('-');
    }

    let value = core.db.get_param(name).unwrap_or("".to_string());
    println!("declare -{flags} {name}={value}");
}

pub fn declare(core: &mut ShellCore, args: &[String],
               subs: &mut [Substitution]) -> i32 {
    let args = arg::dissolve_options(args);

    if subs.is_empty() {
        print_args_match(core, &args)
    } else if arg::has_option("-p", &args) {
        print_params_match(core, subs)
    } else {
        0
    }
}

pub fn local(core: &mut ShellCore, args: &[String],
             subs: &mut [Substitution]) -> i32 {
    let layer = if core.db.get_layer_num() > 2 {
        core.db.get_layer_num() - 2
    } else {
        let e = &ExecError::ValidOnlyInFunction;
        return super::error_exit(1, &args[0], e, core);
    };

    for sub in subs.iter_mut() {
        if let Err(e) = sub.eval(core, Some(layer)) {
            return super::error_exit(1, &args[0], &e, core);
        }
    }

    0
}

pub fn readonly(core: &mut ShellCore, args: &[String],
                subs: &mut [Substitution]) -> i32 {
    for sub in subs.iter_mut() {
        let layer = core.db.solve_set_layer(&sub.left_hand.text, None);
        if let Err(e) = sub.eval(core, Some(layer)) {
            return super::error_exit(1, &args[0], &e, core);
        }

        core.db.set_flag(&sub.left_hand.text, 'r', layer);
    }

    0
}
