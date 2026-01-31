//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

mod database_setter_backend;

use std::env;
use super::{ArrayData, Data, IntData, SingleData, Uninit};
use crate::core::DataBase;
use crate::error::exec::ExecError;

impl DataBase {
    pub fn set_param(
        &mut self,
        name: &str,
        val: &str,
        scope: Option<usize>,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &Some(vec![val.to_string()]))?;

        if name == "BASH_ARGV0" {
            let n = scope.unwrap_or(self.get_scope_num() - 1);
            self.position_parameters[n][0] = val.to_string();
        }

        if !self.flags.contains('r') && (self.flags.contains('a') || self.has_flag(name, 'x')) {
            unsafe{env::set_var(name, "")};
        }

        let scope = self.get_target_scope(name, scope);

        if self.params[scope].get(name).is_none() {
            self.set_entry(scope, name, Box::new(SingleData::from("")))?;
        }

        let d = self.params[scope].get_mut(name).unwrap();
        if let Some(init_d) = d.initialize() {
            *d = init_d;
        }

        d.set_as_single(name, val)?;

        if env::var(name).is_ok() {
            let v = d.get_as_single()?;
            unsafe{env::set_var(name, &v)};
        }
        Ok(())
    }

    pub fn set_param2(
        &mut self,
        name: &str,
        index: &str,
        val: &str,
        scope: Option<usize>,
    ) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.set_param(name, val, scope);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<isize>() {
                self.set_array_elem(name, val, n, scope, false)?;
            }
        } else if self.is_assoc(name) {
            self.set_assoc_elem(name, index, val, scope)?;
        } else {
            match index.parse::<isize>() {
                Ok(n) => self.set_array_elem(name, val, n, scope, false)?,
                _ => self.set_assoc_elem(name, index, val, scope)?,
            }
        }
        Ok(())
    }

    pub fn set_array_elem(&mut self, name: &str, val: &str,
        pos: isize, scope: Option<usize>, append: bool,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &Some(vec![val.to_string()]))?;

        let scope = self.get_target_scope(name, scope);
        let i_flag = self.has_flag(name, 'i');
        match append {
            false => self.set_elem(scope, name, pos, &val.to_string(), i_flag),
            true  => self.append_elem(scope, name, pos, &val.to_string()),
        }
    }

    pub fn set_assoc_elem(&mut self, name: &str, key: &str,
        val: &str, scope: Option<usize>,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &Some(vec![val.to_string()]))?;

        let scope = self.get_target_scope(name, scope);
        match self.params[scope].get_mut(name) {
            Some(v) => {
                if let Some(init_v) = v.initialize() {
                    *v = init_v;
                }
                v.set_as_assoc(name, key, val)
            }
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }

    pub fn set_flag(&mut self, name: &str, flag: char, scope: usize) {
        if let Ok(Some(nameref)) = self.get_nameref(name) {
            return self.set_flag(&nameref, flag, scope);
        }

        match self.params[scope].get_mut(name) {
            Some(d) => { d.set_flag(flag); },
            None => {
                let obj = match flag {
                    'i' => Box::new(IntData::new()) as Box::<dyn Data>,
                    _ => Box::new(Uninit::new(&flag.to_string())) as Box::<dyn Data>,
                };
                let _ = self.set_entry(scope, name, obj);
            }
        }
    }

    pub fn set_scope_to_env(&mut self, scope: usize) {
        for (k, v) in &mut self.params[scope] {
            unsafe{env::set_var(k, v.get_as_single().unwrap_or("".to_string()))};
        }
    }
}
