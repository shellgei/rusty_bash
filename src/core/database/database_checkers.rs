//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::error::exec::ExecError;
use crate::utils;
use crate::utils::restricted_shell;

impl DataBase {
    pub(super) fn check_on_write(&mut self, name: &str, values: &Option<Vec<String>>
    ) -> Result<(), ExecError> {
        Self::name_check(name)?;
        restricted_shell::check(self, name, values)?;
        Ok(())
    }

    pub fn has_array_value(&mut self, name: &str, index: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => d.has_key(index).unwrap_or(false),
            None => false,
        }
    }

    pub fn has_flag_scope(&mut self, name: &str, flag: char, scope: usize) -> bool {
        if let Some(e) = self.params[scope].get_mut(name) {
            return e.has_flag(flag);
        }
        false
    }

    pub fn has_flag(&mut self, name: &str, flag: char) -> bool {
        match self.get_ref(name) {
            Some(d) => d.has_flag(flag),
            None => false,
        }
    }

    pub fn exist(&mut self, name: &str) -> bool {
        if let Ok(Some(nameref)) = self.get_nameref(name) {
            return self.exist(&nameref);
        }

        if let Ok(n) = name.parse::<usize>() {
            let scope = self.position_parameters.len() - 1;
            return n < self.position_parameters[scope].len();
        }

        let num = self.params.len();
        for scope in (0..num).rev() {
            if self.params[scope].contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn exist_nameref(&mut self, name: &str) -> bool {
        if let Some(d) = self.get_ref(name) {
            return d.has_flag('n');
        }

        false
    }

    pub fn exist_l(&mut self, name: &str, scope: usize) -> bool {
        if scope >= self.params.len() {
            return false;
        }

        self.params[scope].contains_key(name)
    }

    pub fn exist_nameref_l(&mut self, name: &str, scope: usize) -> bool {
        if let Some(d) = self.params[scope].get_mut(name) {
            return d.has_flag('n');
        }

        false
    }

    pub fn has_key(&mut self, name: &str, key: &str) -> Result<bool, ExecError> {
        let num = self.params.len();
        for scope in (0..num).rev() {
            if let Some(e) = self.params[scope].get_mut(name) {
                return e.has_key(key);
            }
        }
        Ok(false)
    }

    pub fn name_check(name: &str) -> Result<(), ExecError> {
        if !utils::is_param(name) {
            return Err(ExecError::VariableInvalid(name.to_string()));
        }
        Ok(())
    }

    pub fn is_readonly(&mut self, name: &str) -> bool {
        self.has_flag(name, 'r')
    }

    pub fn is_int(&mut self, name: &str) -> bool {
        self.has_flag(name, 'i')
    }

    pub fn is_assoc(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => d.is_assoc(),
            None => false,
        }
    }

    pub fn is_single(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => d.is_single(),
            _ => false,
        }
    }

    pub fn is_single_num(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => d.is_single_num(),
            _ => false,
        }
    }

    pub fn is_array(&mut self, name: &str) -> bool {
        match self.get_ref(name) {
            Some(d) => d.is_array(),
            _ => false,
        }
    }
}
