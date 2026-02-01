//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::substitution::Substitution;
use crate::error::exec::ExecError;
use crate::utils::arg;

fn set_options_pre(core: &mut ShellCore, name: &String,
                       scope: usize, args: &[String]) {
    if arg::has_option("-x", args) {
        core.db.set_flag(name, 'x', scope);
    }else if arg::has_option("+x", args) {
        core.db.unset_flag(name, 'x', scope);
    }

    if arg::has_option("-n", args) {
        core.db.set_flag(name, 'n', scope);
    }else if arg::has_option("+n", args) {
        core.db.unset_flag(name, 'n', scope);
    }

    if arg::has_option("-i", args) {
        core.db.set_flag(name, 'i', scope);
    }else if arg::has_option("+i", args) {
        core.db.unset_flag(name, 'i', scope);
    }

    if arg::has_option("-l", args) {
        core.db.unset_flag(name, 'u', scope);
        core.db.set_flag(name, 'l', scope);
    }else if arg::has_option("+l", args) {
        core.db.unset_flag(name, 'l', scope);
    }

    if arg::has_option("-u", args) {
        core.db.unset_flag(name, 'l', scope);
        core.db.set_flag(name, 'u', scope);
    }else if arg::has_option("+u", args) {
        core.db.unset_flag(name, 'u', scope);
    }
}

fn set_options_post(core: &mut ShellCore, name: &String,
                       scope: usize, args: &[String]) {
    if arg::has_option("-r", args) {
        core.db.set_flag(&name, 'r', scope);
    }
}

fn readonly_check(core: &mut ShellCore, name: &str) -> Result<(), ExecError> {
    if core.db.is_readonly(&name) {
        return Err(ExecError::VariableReadOnly(name.to_string()));
    }
    Ok(())
}

fn array_to_element_check(sub: &mut Substitution) -> Result<(), ExecError> {
    if let Some(r) = sub.right_hand.as_mut() {
        if sub.left_hand.index.is_some()
        && r.text.starts_with("(") {
            let msg = format!("{}: cannot assign list to array member", sub.left_hand.text);
            return Err(ExecError::Other(msg));
        }
    }
    Ok(())
}

fn check_global_option(core: &mut ShellCore, args: &[String],
                       name: &str, scope: usize) -> usize {
    if arg::has_option("-g", args) && scope != 0 {
        let _ = core.db.unset(&name, None, false);
        return 0;
    }
    scope
}

fn eval(core: &mut ShellCore, args: &[String], sub: &mut Substitution,
        name: &str, scope: usize) -> Result<(), ExecError> {
    if sub.right_hand.is_some() {
        return sub.eval(core, Some(scope), true);
    }

    let change_type = (!core.db.is_array(&name) && arg::has_option("-a", args))
                    || (!core.db.is_assoc(&name) && arg::has_option("-A", args));

    if !core.db.exist_l(&name, scope) || change_type {
        sub.left_hand.init_variable(core, Some(scope), &mut args.to_vec())?;
    }


    Ok(())
}

pub(super) fn exec(core: &mut ShellCore, sub: &mut Substitution, args: &[String],
               scope: usize) -> Result<(), ExecError> {
    let name = sub.left_hand.name.clone();
    readonly_check(core, &name)?;
    array_to_element_check(sub)?;
    let scope = check_global_option(core, args, &name, scope);

    if arg::has_option("+i", args) && core.db.has_flag_scope(&name, 'i', scope) {
        core.db.int_to_str_type(&name, scope)?;
    }

    let arg_indicate_array = arg::has_option("-A", args) || arg::has_option("-a", args);

    if arg_indicate_array && !core.db.exist(&name) && !core.db.exist_nameref(&name) {
        sub.left_hand.init_variable(core, Some(scope), &mut args.to_vec())?;
    }

    if let Some(r) = sub.right_hand.as_mut() {
        let right_is_array = ["(", "'(", "\"("].iter().any(|e| r.text.starts_with(e));
        if arg_indicate_array && right_is_array {
            sub.left_hand.index = None;
        }
    }

    let already_array = core.db.is_array(&name) || core.db.is_assoc(&name);
    let subs_elem_quoted_string = match sub.right_hand.as_mut() {
        Some(r) => sub.left_hand.index.is_some()
                   && (r.text.starts_with("'") || r.text.starts_with("\"")),
        _ => false,
    };

    if arg_indicate_array || (already_array && args[0] == "declare") {
        sub.quoted = false;   //^ Bash bug???
    }

    let treat_as_export = core.db.has_flag(&name, 'x') || arg::has_option("-x", args);
    if sub.right_hand.is_some()
        && already_array
        && !subs_elem_quoted_string
        && (!treat_as_export || arg_indicate_array) {
        sub.reparse(core)?;
    }

    set_options_pre(core, &name, scope, args);
    let res = eval(core, args, sub, &name, scope);
    set_options_post(core, &name, scope, args);

    res
}
