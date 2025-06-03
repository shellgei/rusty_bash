//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, utils};
use super::SimpleCommand;
use crate::elements::command::ExecError;

pub fn hash_control(com: &mut SimpleCommand, core: &mut ShellCore)
-> Result<String, ExecError> {
    if com.args[0].starts_with("/")
    || com.args[0].starts_with("./")
    || com.args[0].starts_with("../") {
        return Ok(com.args[0].clone());
    }

    let hash = core.db.get_elem("BASH_CMDS", &com.args[0])?;

    let restricted = core.db.flags.contains('r');
    core.db.flags.retain(|f| f != 'r');
    if hash.is_empty() {
        let path = utils::get_command_path(&com.args[0], core);
        if path != "" {
            core.db.set_assoc_elem("BASH_CMDS", &com.args[0], &path, None)?;
            core.db.hash_counter.insert(com.args[0].clone(), 1);
            if restricted {
                core.db.flags.push('r');
            }
            return Ok(path);
        }
    }else{
        match core.db.hash_counter.get_mut(&com.args[0]) {
            Some(v) => *v += 1,
            None => {core.db.hash_counter.insert(com.args[0].clone(), 1);},
        }
        if restricted {
            core.db.flags.push('r');
        }
        return Ok(hash);
    }
    if restricted {
        core.db.flags.push('r');
    }
    Ok(String::new())
}
