//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod print;
mod set_value;

use crate::elements::substitution::Substitution;
use crate::error::exec::ExecError;
use crate::utils::arg;
use crate::{env, ShellCore};

pub fn local(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let args = args.to_owned();
    let layer = if core.db.get_layer_num() > 2 {
        core.db.get_layer_num() - 2 //The last element of data.parameters is for local itself.
    } else {
        let e = &ExecError::ValidOnlyInFunction;
        return super::error_exit(1, &args[0], e, core);
    };

    if core.shopts.query("localvar_inherit") {
        subs.into_iter().for_each(|e| e.localvar_inherit(core) );
    }

    for sub in subs.iter_mut() {
        if let Err(e) = set_value::exec(core, sub, &args, layer) {
            e.print(core);
            return 1;
        }
    }

    0
}

pub fn declare(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let mut args = arg::dissolve_options(args);

    if subs.is_empty() {
        return print::all_match(core, &mut args);
    }

    if arg::has_option("-f", &args) {
        return print::functions(core, &args, subs);
    }

    if arg::consume_arg("-p", &mut args) {
        let mut names = vec![];
        for sub in subs {
            names.push(sub.text.clone());
        }
        return print::params(core, &names, &args);
    }

    let layer = core.db.get_layer_num() - 2;
    for sub in subs {
        if let Err(e) = set_value::exec(core, sub, &args, layer) {
            return super::error_exit(1, &args[0], &e, core);
        }
    }
    0
}

pub fn export(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let mut args = args.to_owned();
    for sub in subs.iter_mut() {
        let layer = core.db.get_layer_pos(&sub.left_hand.name).unwrap_or(0);
        args.push("-x".to_string());
        if let Err(e) = set_value::exec(core, sub, &args, layer) {
            e.print(core);
            return 1;
        }
        match core.db.get_param(&sub.left_hand.name) {
            Ok(v) => unsafe{env::set_var(&sub.left_hand.name, v)},
            Err(e) => {
                e.print(core);
                return 1;
            }
        }
    }
    0
}

pub fn readonly(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let args = arg::dissolve_options(args);

    if subs.is_empty() {
        let mut args = args.to_vec();
        args.push("-r".to_string()); 
        return print::all_match(core, &mut args);
    }

    for sub in subs {
        if sub.left_hand.index.is_some() {
            let e = ExecError::VariableInvalid(sub.left_hand.text.clone());
            return super::error_exit(1, &args[0], &e, core);
        }

        let layer = core.db.get_layer_pos(&sub.left_hand.name).unwrap_or(0);

        if let Err(e) = set_value::exec(core, sub, &args, layer) {
            return super::error_exit(1, &args[0], &e, core);
        }
        core.db.set_flag(&sub.left_hand.name, 'r', layer);
    }
    0
}
