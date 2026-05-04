//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::env;
use super::SingleData;
use crate::core::DataBase;
use crate::error::exec::ExecError;

impl DataBase {
    pub fn append_param(
        &mut self,
        name: &str,
        val: &str,
        scope: Option<usize>,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &Some(vec![val.to_string()]))?;

        /*
        if let Some(nameref) = self.get_nameref(name)? {
            return self.set_param(&nameref, val, scope);
        }*/

        if name == "BASH_ARGV0" {
            let n = scope.unwrap_or(self.get_scope_num() - 1);
            self.position_parameters[n][0] += val;
        }

        if !self.flags.contains('r')
            && (self.flags.contains('a') || self.has_flag(name, 'x'))
            && env::var(name).is_err()
        {
            unsafe{env::set_var(name, "")};
        }

        let scope = self.get_target_scope(name, scope);

        if self.params[scope].get(name).is_none() {
            self.set_entry(scope, name, Box::new(SingleData::from("")))?;
        }

        let d = self.params[scope].get_mut(name).unwrap();
        if d.is_array() {
            return d.append_to_array_elem(name, "0", val);
        }

        d.append_as_single(name, val)?;

        if env::var(name).is_ok() {
            let v = d.get_as_single()?;
            unsafe{env::set_var(name, v)};
        }

        Ok(())
    }

    pub fn append_param2(
        &mut self,
        name: &str,
        index: &str,
        val: &str,
        scope: Option<usize>,
    ) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.append_param(name, val, scope);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<isize>() {
                self.set_array_elem(name, val, n, scope, true)?;
            }
        } else if self.is_assoc(name) {
            self.append_to_assoc_elem(name, index, val, scope)?;
        } else {
            match index.parse::<isize>() {
                Ok(n) => self.set_array_elem(name, val, n, scope, true)?,
                _ => self.append_to_assoc_elem(name, index, val, scope)?,
            }
        }
        Ok(())
    }

    pub fn append_to_assoc_elem(&mut self, name: &str, key: &str,
        val: &str, scope: Option<usize>,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &Some(vec![val.to_string()]))?;

        let scope = self.get_target_scope(name, scope);
        match self.params[scope].get_mut(name) {
            Some(v) => v.append_to_assoc_elem(name, key, val),
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }
}
