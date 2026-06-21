//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::core::database::Uninit;
use crate::error::exec::ExecError;
//use std::env;

impl DataBase {
    pub fn unset_flag(&mut self, name: &str, flag: char, scope: usize) {
        if flag != 'n'
            && let Ok(Some(nameref)) = self.get_nameref(name)
        {
            return self.unset_flag(&nameref, flag, scope);
        }

        let rf = &mut self.params[scope];
        if let Some(d) = rf.get_mut(name) {
            d.unset_flag(flag);
        }
    }

    pub fn unset_flag_nameref(&mut self, name: &str, flag: char, scope: usize) {
        let rf = &mut self.params[scope];
        if let Some(d) = rf.get_mut(name) {
            d.unset_flag(flag);
        }
    }

    pub fn unset_nameref(
        &mut self,
        name: &str,
        called_scope: Option<usize>,
    ) -> Result<(), ExecError> {
        if let Some(scope) = called_scope {
            if let Some(d) = self.params[scope].get_mut(name)
                && d.has_flag('n')
            {
                self.remove_entry(scope, name)?;
            }
            return Ok(());
        }

        let num = self.params.len();
        for scope in 0..num {
            if let Some(d) = self.params[scope].get_mut(name)
                && d.has_flag('n')
            {
                self.remove_entry(scope, name)?;
            }
        }
        Ok(())
    }

    pub fn unset_var(
        &mut self,
        name: &str,
        called_scope: Option<usize>,
        localvar_unset: bool,
    ) -> Result<bool, ExecError> {
        if let Ok(Some(nameref)) = self.get_nameref(name) {
            if !nameref.is_empty() {
                return self.unset_var(&nameref, called_scope, localvar_unset);
            }
            return Ok(false);
        }

        if called_scope.is_none() || called_scope.unwrap() == 0 {
            return self.unset_surface_scope_var(name);
        }

        let mut res = false;
        for scope in self.params.iter_mut() {
            if let Some(d) = scope.get_mut(name) {
                res = true;
                if localvar_unset {
                    *d = Box::new(Uninit::new(""));
                    break;
                }

                scope.remove(name);
            }
        }

        Ok(res)
    }

    fn unset_surface_scope_var(&mut self, name: &str)
    -> Result<bool, ExecError> {
        let num = self.params.len();
        for scope in (0..num).rev() {
            if self.remove_entry(scope, name)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn unset_function(&mut self, name: &str) {
        self.functions.remove(name);
    }

    pub fn unset(
        &mut self,
        name: &str,
        called_scope: Option<usize>,
        localvar_unset: bool,
    ) -> Result<(), ExecError> {
        if self.unset_var(name, called_scope, localvar_unset)? {
            return Ok(());
        }
        self.unset_function(name);
        Ok(())
    }

    pub fn unset_array_elem(&mut self, name: &str, key: &str) -> Result<(), ExecError> {
        if self.is_single(name) && (key == "0" || key == "@" || key == "*") {
            self.unset_var(name, None, false)?;
            return Ok(());
        }

        for scope in &mut self.params {
            if let Some(d) = scope.get_mut(name) {
                d.remove_elem(key)?;
            }
        }
        Ok(())
    }
}
