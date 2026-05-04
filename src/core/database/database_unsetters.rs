//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::core::database::Uninit;
use crate::error::exec::ExecError;
use std::env;

impl DataBase {
    pub fn unset_flag(&mut self, name: &str, flag: char, scope: usize) {
        if flag != 'n' {
            if let Ok(Some(nameref)) = self.get_nameref(name) {
                return self.unset_flag(&nameref, flag, scope);
            }
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

    pub fn unset_nameref(&mut self, name: &str,
                         called_scope: Option<usize>) -> Result<(), ExecError> {
        if let Some(scope) = called_scope {
            if let Some(d) = self.params[scope].get_mut(name) {
                if d.has_flag('n') {
                    self.remove_entry(scope, name)?;
                }
            }
            return Ok(());
        }

        let num = self.params.len();
        for scope in 0..num {
            if let Some(d) = self.params[scope].get_mut(name) {
                if d.has_flag('n') {
                    self.remove_entry(scope, name)?;
                }
            }
        }
        Ok(())
    }

    pub fn unset_var(&mut self, name: &str,
                     called_scope: Option<usize>,
                     localvar_unset: bool) -> Result<bool, ExecError> {
        if let Ok(Some(nameref)) = self.get_nameref(name) {
            if nameref != "" {
                 return self.unset_var(&nameref, called_scope, localvar_unset);
            }
            return Ok(false);
        }

        let mut res = false;

        if let Some(scope) = called_scope {
            if scope == 0 {
                return Ok(false);
            }
            res = self.remove_entry(scope, name)?;

            unsafe{env::set_var(name, "")};
            for scope in self.params.iter_mut() {
                if let Some(d) = scope.get_mut(name) {
                    res = true;
                    if localvar_unset {
                        *d = Box::new( Uninit::new("") );
                    }else {
                        scope.remove(name);
                    }
                }
            }

            return Ok(res);
        }

        let num = self.params.len();
        for scope in (0..num).rev() {
            if self.remove_entry(scope, name)? {
                return Ok(true);
            }
        }
        Ok(res)
    }

    pub fn unset_function(&mut self, name: &str) {
        self.functions.remove(name);
    }

    pub fn unset(&mut self, name: &str, called_scope: Option<usize>,
                 localvar_unset: bool) -> Result<(), ExecError> {
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
