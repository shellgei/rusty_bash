//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{SimpleCommand, SubsArgType};
use crate::error::exec::ExecError;
use crate::ShellCore;

pub fn run(com: &mut SimpleCommand, core: &mut ShellCore) -> Result<bool, ExecError> {
    let ans = run_function(&mut com.args, core)
        || run_substitution_builtin(com, core)?
        || run_builtin(com, core)?;
    Ok(ans)
}

fn run_function(args: &mut [String], core: &mut ShellCore) -> bool {
    match core.db.functions.get_mut(&args[0]) {
        Some(f) => {
            f.clone().run_as_command(args, core);
            true
        }
        None => false,
    }
}

pub fn run_builtin(com: &mut SimpleCommand, core: &mut ShellCore) -> Result<bool, ExecError> {
    if com.args.is_empty() {
        eprintln!("ShellCore::run_builtin");
        return Ok(false);
    }

    if !core.builtins.contains_key(&com.args[0]) {
        return Ok(false);
    }

    let func = core.builtins[&com.args[0]];
    let exit_status = func(core, &com.args[..]);
    let _ = core.db.set_param("?", &exit_status.to_string(), Some(0));
    Ok(true)
}

pub fn run_substitution_builtin(
    com: &mut SimpleCommand,
    core: &mut ShellCore,
) -> Result<bool, ExecError> {
    if !core.subst_builtins.contains_key(&com.args[0]) {
        return Ok(false);
    }

    let mut args = vec![com.args[0].clone()];
    let mut subs = vec![];
    for sub in com.substitutions_as_args.iter_mut() {
        match sub {
            SubsArgType::Subs(s) => subs.push((**s).clone()),
            SubsArgType::Other(w) => {},
        }
    }

    let func = core.subst_builtins[&com.args[0]];
    let exit_status = func(core, &args[..], &mut subs[..]);
    core.db.set_param("?", &exit_status.to_string(), None)?;
    Ok(true)
}
