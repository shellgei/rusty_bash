//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::error_exit;
use crate::elements::substitution::Substitution;
use crate::error::exec::ExecError;
use crate::utils::arg;
use crate::{env, ShellCore};

pub fn local(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let args = args.to_owned();
    let layer = if core.db.get_layer_num() > 2 {
        core.db.get_layer_num() - 2 //The last element of data.parameters is for local itself.
    } else {
        ExecError::ValidOnlyInFunction("local".to_string()).print(core);
        return 1;
    };

    for sub in subs.iter_mut() {
        if let Err(e) = set_substitution(core, sub, &args, layer) {
            e.print(core);
            return 1;
        }
    }

    0
}

fn set_substitution(
    core: &mut ShellCore,
    sub: &mut Substitution,
    args: &[String],
    layer: usize,
) -> Result<(), ExecError> {
    if core.db.is_readonly(&sub.left_hand.name) {
        return Err(ExecError::VariableReadOnly(sub.left_hand.name.clone()));
    }

    if sub.left_hand.index.is_some() && sub.right_hand.text.starts_with("(") {
        let msg = format!("{}: cannot assign list to array member", sub.left_hand.text);
        return Err(ExecError::Other(msg));
    }

    let read_only = arg::has_option("-r", args);
    let export_opt = arg::has_option("-x", args);
    let little_opt = arg::has_option("-l", args);
    let upper_opt = arg::has_option("-u", args);

    let mut layer = layer;
    if arg::has_option("-g", args) && layer != 0 {
        core.db.unset(&sub.left_hand.name);
        layer = 0;
    }

    if arg::has_option("+i", args) && core.db.has_flag_layer(&sub.left_hand.name, 'i', layer) {
        core.db.int_to_str_type(&sub.left_hand.name, layer)?;
    }

    if (arg::has_option("-A", args) || arg::has_option("-a", args))
        && !core.db.exist(&sub.left_hand.name)
    {
        let mut args_clone = args.to_vec();
        sub.left_hand
            .init_variable(core, Some(layer), &mut args_clone)?;
    }

    if (arg::has_option("-A", args) || arg::has_option("-a", args))
        && (sub.right_hand.text.starts_with("(") || sub.right_hand.text.starts_with("'("))
    {
        sub.left_hand.index = None;
    }

    if sub.has_right
        && (core.db.is_array(&sub.left_hand.name) || core.db.is_assoc(&sub.left_hand.name))
        && !(sub.left_hand.index.is_some() && sub.right_hand.text.starts_with("'"))
    {
        sub.reparse(core)?;
    }

    if export_opt {
        core.db.set_flag(&sub.left_hand.name, 'x', Some(layer));
    }

    if arg::has_option("-i", args) {
        core.db.set_flag(&sub.left_hand.name, 'i', Some(layer));
    }

    if little_opt {
        core.db.unset_flag(&sub.left_hand.name, 'u', Some(layer));
        core.db.set_flag(&sub.left_hand.name, 'l', Some(layer));
    }

    if upper_opt {
        core.db.unset_flag(&sub.left_hand.name, 'l', Some(layer));
        core.db.set_flag(&sub.left_hand.name, 'u', Some(layer));
    }

    let mut res = Ok(());

    match sub.has_right {
        true => res = sub.eval(core, Some(layer), true),
        false => {
            if !core.db.params[layer].contains_key(&sub.left_hand.name)
                || (!core.db.is_array(&sub.left_hand.name) && arg::has_option("-a", args))
                || (!core.db.is_assoc(&sub.left_hand.name) && arg::has_option("-A", args))
            {
                let mut args_clone = args.to_vec();
                res = sub
                    .left_hand
                    .init_variable(core, Some(layer), &mut args_clone);
            }
        }
    }

    if read_only {
        core.db.set_flag(&sub.left_hand.name, 'r', Some(layer));
    }

    res
}

fn declare_print(core: &mut ShellCore, names: &[String], com: &str) -> i32 {
    for n in names {
        let mut opt = if core.db.is_assoc(n) {
            "A"
        } else if core.db.is_array(n) {
            "a"
        } else if core.db.exist(n) {
            ""
        } else {
            return error_exit(1, n, "not found", core);
        }
        .to_string();

        if core.db.is_int(n) {
            opt += "i";
        }
        if core.db.has_flag(n, 'l') {
            opt += "l";
        }
        if core.db.has_flag(n, 'u') {
            opt += "u";
        }

        if core.db.is_readonly(n) && !opt.contains('r') && !core.options.query("posix") {
            opt += "r";
        }

        if opt.is_empty() {
            opt += "-";
        }

        let prefix = match core.options.query("posix") {
            false => format!("declare -{opt} "),
            true => format!("{com} -{opt} "),
        };
        print!("{prefix}");
        core.db.declare_print(n);
    }
    0
}

fn declare_print_all(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() < 2 {
        core.db
            .get_keys()
            .into_iter()
            .for_each(|k| core.db.print(&k));
        return 0;
    }

    if args.len() == 2 && args[1] == "-f" {
        let mut names: Vec<String> = core.db.functions.keys().map(|k| k.to_string()).collect();
        names.sort();

        for n in names {
            core.db.functions.get_mut(&n).unwrap().pretty_print(0);
        }
        return 0;
    }

    let mut names = core.db.get_keys();
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
        if core.db.is_readonly(&name) && !options.contains('r') && !core.options.query("posix") {
            print!("r");
        }
        print!(" ");
        core.db.declare_print(&name);
    }
    0
}

fn declare_print_function(core: &mut ShellCore, subs: &mut [Substitution]) -> i32 {
    let mut names: Vec<String> = subs.iter().map(|s| s.left_hand.name.clone()).collect();
    names.sort();

    for n in &names {
        if n.is_empty() {
            return 1;
        }

        match core.db.functions.get_mut(n) {
            Some(f) => f.pretty_print(0),
            None => return 1,
        }
    }
    0
}

pub fn declare(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let args = arg::dissolve_options(args);

    if args[1..].iter().all(|a| a.starts_with("-")) && subs.is_empty() {
        return declare_print_all(core, &args);
    }

    if arg::has_option("-f", &args) {
        return declare_print_function(core, subs);
    }

    if arg::has_option("-p", &args) {
        let mut print_args = args.to_vec();
        for sub in subs {
            print_args.push(sub.text.clone());
        }
        return declare_print(core, &print_args[1..], &print_args[0]);
    }

    let layer = core.db.get_layer_num() - 2;
    for sub in subs {
        if let Err(e) = set_substitution(core, sub, &args, layer) {
            return super::error_exit(1, &args[0], &String::from(&e), core);
        }
    }
    0
}

pub fn export(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let args = args.to_owned();
    for sub in subs.iter_mut() {
        let layer = core.db.get_layer_pos(&sub.left_hand.name).unwrap_or(0);
        if let Err(e) = set_substitution(core, sub, &args, layer) {
            e.print(core);
            return 1;
        }
        match core.db.get_param(&sub.left_hand.name) {
            Ok(v) => env::set_var(&sub.left_hand.name, v),
            Err(e) => {
                e.print(core);
                return 1;
            }
        }
    }
    0
}

pub fn readonly_print(core: &mut ShellCore, args: &mut [String]) -> i32 {
    let array_opt = arg::has_option("-a", args);
    let assoc_opt = arg::has_option("-A", args);
    let int_opt = arg::has_option("-i", args);

    let mut names: Vec<String> = core
        .db
        .get_keys()
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

    declare_print(core, &names, &args[0]);
    0
}

pub fn readonly(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let args = arg::dissolve_options(args);

    if subs.is_empty() {
        let mut args_mut = args;
        return readonly_print(core, &mut args_mut);
    }

    for sub in subs {
        if sub.left_hand.index.is_some() {
            let msg = ExecError::VariableInvalid(sub.left_hand.text.clone());
            return super::error_exit(1, &args[0], &String::from(&msg), core);
            //return 1;
        }

        let layer = core.db.get_layer_pos(&sub.left_hand.name).unwrap_or(0);

        if let Err(e) = set_substitution(core, sub, &args, layer) {
            e.print(core);
            return 1;
        }
        core.db.set_flag(&sub.left_hand.name, 'r', Some(layer));
    }
    0
}
