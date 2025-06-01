//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils;
use crate::utils::file_check;
use crate::error::exec::ExecError;
use crate::core::DataBase;

impl DataBase {
    pub fn name_check(name: &str) -> Result<(), ExecError> {
        if ! utils::is_param(name) {
            return Err(ExecError::VariableInvalid(name.to_string()));
        }
        Ok(())
    }

    pub fn is_readonly(&mut self, name: &str) -> bool {
        self.has_flag(name, 'r')
    }

    pub fn rsh_cmd_check(&mut self, cmds: &Vec<String>) -> Result<(), ExecError> {
        for c in cmds {
            if c.contains('/') {
                let msg = format!("{}: restricted", &c);
                return Err(ExecError::Other(msg));
            }

            if file_check::is_executable(&c) {
                let msg = format!("{}: not found", &c);
                return Err(ExecError::Other(msg));
            }
        }
        Ok(())
    }

    pub fn rsh_check(&mut self, name: &str) -> Result<(), ExecError> {
        if ["SHELL", "PATH", "ENV", "BASH_ENV"].contains(&name) {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        Ok(())
    }

    pub fn write_check(&mut self, name: &str) -> Result<(), ExecError> {
        if self.has_flag(name, 'r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        Ok(())
    }
}
