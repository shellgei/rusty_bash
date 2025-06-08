//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, utils};
use super::SimpleCommand;
use crate::elements::command::ExecError;

pub fn get_and_regist(com: &mut SimpleCommand, core: &mut ShellCore)
-> Result<String, ExecError> {
    if ["/", "./", "../"].iter().any(|p| com.args[0].starts_with(p)) {
        return Ok(com.args[0].clone());
    }

    let mut path = core.db.get_elem("BASH_CMDS", &com.args[0])?;

    if path.is_empty() {
        path = resolve_path(&com.args[0], core)?;
    }

    count_up(&com.args[0], core);
    Ok(path)
}

fn resolve_path(arg: &String, core: &mut ShellCore)
-> Result<String, ExecError> {
    let path = utils::get_command_path(arg, core);
    if path.is_empty() {
        return Ok(path);
    }

    let restricted = core.db.flags.contains('r');
    core.db.flags.retain(|f| f != 'r');
    core.db.set_assoc_elem("BASH_CMDS", arg, &path, None)?;
    if restricted {
        core.db.flags.push('r');
    }

    Ok(path)
}

fn count_up(arg: &String, core: &mut ShellCore) {
    match core.db.hash_counter.get_mut(arg) {
        Some(v) => *v += 1,
        None => {core.db.hash_counter.insert(arg.to_string(), 1);},
    }
}
