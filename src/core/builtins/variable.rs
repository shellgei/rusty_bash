//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod print;

use crate::core::database::DataBase;
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

    if let Some(r) = sub.right_hand.as_mut() {
        if sub.left_hand.index.is_some()
        && r.text.starts_with("(") {
            let msg = format!("{}: cannot assign list to array member", sub.left_hand.text);
            return Err(ExecError::Other(msg));
        }
    }

    let read_only = arg::has_option("-r", args);
    let export_opt = arg::has_option("-x", args);
    let little_opt = arg::has_option("-l", args);
    let upper_opt = arg::has_option("-u", args);
    let nameref_opt = arg::has_option("-n", args);

    let mut layer = layer;
    if arg::has_option("-g", args) && layer != 0 {
        core.db.unset(&sub.left_hand.name, None)?;
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

    if let Some(r) = sub.right_hand.as_mut() {
        if (arg::has_option("-A", args) || arg::has_option("-a", args))
            && (r.text.starts_with("(") || r.text.starts_with("'(")) {
            sub.left_hand.index = None;
        }
    }

    /* TODO: chaos!!!! */
    let treat_as_array =
        core.db.is_array(&sub.left_hand.name) || core.db.is_assoc(&sub.left_hand.name);
    let option_indicate_array = arg::has_option("-A", args) || arg::has_option("-a", args);
    let treat_as_export = core.db.has_flag(&sub.left_hand.name, 'x') || export_opt;
    let subs_elem_quoted_string = match sub.right_hand.as_mut() {
        Some(r) => sub.left_hand.index.is_some() && r.text.starts_with("'"),
        _ => false,
    };

    if option_indicate_array {
        sub.quoted = false;
    }

    if sub.right_hand.is_some()
        && treat_as_array
        && !subs_elem_quoted_string
        && (!treat_as_export || option_indicate_array)
    {
        sub.reparse(core)?;
    }

    if export_opt {
        core.db.set_flag(&sub.left_hand.name, 'x', layer);
    }

    if nameref_opt {
        core.db.set_flag(&sub.left_hand.name, 'n', layer);
    }

    if arg::has_option("-i", args) {
        core.db.set_flag(&sub.left_hand.name, 'i', layer);
    }

    if little_opt {
        core.db.unset_flag(&sub.left_hand.name, 'u', layer);
        core.db.set_flag(&sub.left_hand.name, 'l', layer);
    }

    if upper_opt {
        core.db.unset_flag(&sub.left_hand.name, 'l', layer);
        core.db.set_flag(&sub.left_hand.name, 'u', layer);
    }

    let mut res = Ok(());

    match sub.right_hand.is_some() {
        true => res = sub.eval(core, Some(layer), true),
        false => {
            //if !core.db.params[layer].contains_key(&sub.left_hand.name)
            if !core.db.exist_l(&sub.left_hand.name, layer)
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
        core.db.set_flag(&sub.left_hand.name, 'r', layer);
    }

    res
}

pub fn declare(core: &mut ShellCore, args: &[String], subs: &mut [Substitution]) -> i32 {
    let mut args = arg::dissolve_options(args);

    if args.len() <= 1 && subs.is_empty() {
        DataBase::print_params_and_funcs(core);
        return 0;//p_optionrint_all(core);
    }

    if args[1..].iter().all(|a| a.starts_with("-")) && subs.is_empty() {
        return print::declare_print(core, &args);
    }

    if arg::has_option("-f", &args) {
        return print::functions(core, subs);
    }

    if arg::consume_arg("-p", &mut args) {
        let mut print_args = args.to_vec();
        for sub in subs {
            print_args.push(sub.text.clone());
        }
        return print::p_option(core, &print_args[1..], &args[0]);
    }

    let layer = core.db.get_layer_num() - 2;
    for sub in subs {
        if let Err(e) = set_substitution(core, sub, &args, layer) {
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
        if let Err(e) = set_substitution(core, sub, &args, layer) {
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
        let mut args_mut = args;
        return print::readonly_params(core, &mut args_mut);
    }

    for sub in subs {
        if sub.left_hand.index.is_some() {
            let e = ExecError::VariableInvalid(sub.left_hand.text.clone());
            return super::error_exit(1, &args[0], &e, core);
        }

        let layer = core.db.get_layer_pos(&sub.left_hand.name).unwrap_or(0);

        if let Err(e) = set_substitution(core, sub, &args, layer) {
            return super::error_exit(1, &args[0], &e, core);
        }
        core.db.set_flag(&sub.left_hand.name, 'r', layer);
    }
    0
}
