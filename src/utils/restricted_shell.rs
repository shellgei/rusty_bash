//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::database::DataBase;
use crate::error::exec::ExecError;
use crate::file_check;

pub fn check(db: &mut DataBase, name: &str, value: &Option<Vec<String>>) -> Result<(), ExecError> {
    if !db.flags.contains('r') {
        return Ok(());
    }

    if value.is_some() && name == "BASH_CMDS" {
        rsh_cmd_check(value.as_ref().unwrap())?;
    }

    if ["SHELL", "PATH", "ENV", "BASH_ENV"].contains(&name) {
        return Err(ExecError::VariableReadOnly(name.to_string()));
    }
    Ok(())
}

fn rsh_cmd_check(cmds: &Vec<String>) -> Result<(), ExecError> {
    for c in cmds {
        if c.contains('/') {
            let msg = format!("{}: restricted", &c);
            return Err(ExecError::Other(msg));
        }

        if file_check::is_executable(c) {
            let msg = format!("{}: not found", &c);
            return Err(ExecError::Other(msg));
        }
    }
    Ok(())
}
