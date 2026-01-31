//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::core::database::Uninit;
use crate::error::exec::ExecError;
use std::env;

impl DataBase {
    pub fn unset_flag(&mut self, name: &str, flag: char, layer: usize) {
        if let Ok(Some(nameref)) = self.get_nameref(name) {
            return self.unset_flag(&nameref, flag, layer);
        }

        let rf = &mut self.params[layer];
        if let Some(d) = rf.get_mut(name) {
            d.unset_flag(flag);
        }
    }

    pub fn unset_nameref(&mut self, name: &str,
                         called_layer: Option<usize>) -> Result<(), ExecError> {
        if let Some(layer) = called_layer {
            if let Some(d) = self.params[layer].get_mut(name) {
                if d.has_flag('n') {
                    self.remove_entry(layer, name)?;
                }
            }
            return Ok(());
        }

        let num = self.params.len();
        for layer in 0..num {
            if let Some(d) = self.params[layer].get_mut(name) {
                if d.has_flag('n') {
                    self.remove_entry(layer, name)?;
                }
            }
        }
        Ok(())
    }

    pub fn unset_var(&mut self, name: &str,
                     called_layer: Option<usize>,
                     localvar_unset: bool) -> Result<bool, ExecError> {
        if let Ok(Some(nameref)) = self.get_nameref(name) {
            if nameref != "" {
                 return self.unset_var(&nameref, called_layer, localvar_unset);
            }
            return Ok(false);
        }

        let mut res = false;

        if let Some(layer) = called_layer {
            if layer == 0 {
                return Ok(false);
            }
            res = self.remove_entry(layer, name)?;

            unsafe{env::set_var(name, "")};
            for layer in self.params.iter_mut() {
                if let Some(d) = layer.get_mut(name) {
                    res = true;
                    if localvar_unset {
                        *d = Box::new( Uninit::new("") );
                    }else {
                        layer.remove(name);
                    }
                }
            }

            return Ok(res);
        }

        let num = self.params.len();
        for layer in (0..num).rev() {
            if self.remove_entry(layer, name)? {
                return Ok(true);
            }
        }
        Ok(res)
    }

    pub fn unset_function(&mut self, name: &str) {
        self.functions.remove(name);
    }

    pub fn unset(&mut self, name: &str, called_layer: Option<usize>,
                 localvar_unset: bool) -> Result<(), ExecError> {
        if self.unset_var(name, called_layer, localvar_unset)? {
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

        for layer in &mut self.params {
            if let Some(d) = layer.get_mut(name) {
                d.remove_elem(key)?;
            }
        }
        Ok(())
    }
}
