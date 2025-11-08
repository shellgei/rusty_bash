//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::error::exec::ExecError;
use crate::utils;

impl DataBase {
    pub fn has_array_value(&mut self, name: &str, index: &str) -> bool {
        let num = self.params.len();
        for layer in (0..num).rev() {
            if let Some(e) = self.params[layer].get(name) {
                let mut a = e.clone();
                if a.has_key(index).unwrap_or(false) {
                    return true;
                }
                //return a.get_as_array_or_assoc(index, "").is_ok();
            }
        }
        false
    }

    pub fn has_flag_layer(&mut self, name: &str, flag: char, layer: usize) -> bool {
        if let Some(e) = self.params[layer].get_mut(name) {
            return e.has_flag(flag).unwrap_or(false);
        }
        false
    }

    pub fn has_flag(&mut self, name: &str, flag: char) -> bool {
        let num = self.params.len();
        for layer in (0..num).rev() {
            if let Some(e) = self.params[layer].get_mut(name) {
                return e.has_flag(flag).unwrap_or(false);
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
        for layer in (0..num).rev() {
            if self.params[layer].contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn has_key(&mut self, name: &str, key: &str) -> Result<bool, ExecError> {
        let num = self.params.len();
        for layer in (0..num).rev() {
            if let Some(e) = self.params[layer].get_mut(name) {
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
