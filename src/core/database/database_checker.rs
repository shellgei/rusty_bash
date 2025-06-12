//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils;
use crate::utils::file_check;
use crate::error::exec::ExecError;
use crate::core::DataBase;

impl DataBase {
    pub fn has_array_value(&mut self, name: &str, index: &str) -> bool {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if let Some(e) = self.params[layer].get(name) {
                let mut a = e.clone();
                return a.get_as_array_or_assoc(index, "").is_ok();
            }
        }
        false
    }

    pub fn has_flag(&mut self, name: &str, flag: char) -> bool {
        let num = self.params.len();
        for layer in (0..num).rev()  {
            if let Some(e) = self.param_options[layer].get(name) {
                return e.contains(flag);
            }
        }
        false
    }

    pub fn exist(&mut self, name: &str) -> bool {
        if let Ok(n) = name.parse::<usize>() {
            let layer = self.position_parameters.len() - 1;
            return n < self.position_parameters[layer].len();
        }

        let num = self.params.len();
        for layer in (0..num).rev()  {
            if self.params[layer].get(name).is_some() {
                return true;
            }
        }
        false
    }

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

    pub fn is_assoc(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => d.is_assoc(),
            None => false,
        }
    }

    pub fn is_single(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => return d.is_single(),
            _ => false,
        }
    }

    pub fn is_single_num(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => return d.is_single_num(),
            _ => false,
        }
    }

    pub fn is_array(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => return d.is_array(),
            _ => false,
        }
    }
}
