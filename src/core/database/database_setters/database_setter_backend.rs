//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::env;
use super::{ArrayData, Data, Uninit};
use crate::core::DataBase;
use crate::error::exec::ExecError;

impl DataBase {
    pub(super) fn set_elem(&mut self, layer: usize, name: &str,
        pos: isize, val: &String, i_flag: bool
        ) -> Result<(), ExecError> {
        if self.is_readonly(name) {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        if ! self.params[layer].contains_key(name) {
            self.set_uninit_array(layer, name, i_flag)?;
            return self.set_elem(layer, name, pos, val, i_flag);
        }


        let d = self.params[layer].get_mut(name).unwrap();
        if let Some(init_d) = d.initialize() {
            *d = init_d;
        }

        if d.is_array() {
            d.set_as_array(name, &pos.to_string(), val)
        } else if d.is_assoc() {
            d.set_as_assoc(name, &pos.to_string(), val)
        } else {
            let data = d.get_as_single()?;
            self.set_uninit_array(layer, name, i_flag)?;

            if !data.is_empty() {
                self.set_elem(layer, name, 0, &data, i_flag)?;
            }
            self.set_elem(layer, name, pos, val, i_flag)
        } 
    }

    pub(super) fn append_elem(&mut self, layer: usize, 
        name: &str, pos: isize, val: &String,
    ) -> Result<(), ExecError> {
        if ! self.params[layer].contains_key(name) {
            self.set_uninit_array(layer, name, false)?;
            return self.set_elem(layer, name, pos, val, false);
        }

        let d = self.params[layer].get_mut(name).unwrap();
        if let Some(init_d) = d.initialize() {
            *d = init_d;
        }

        if d.is_array() {
            d.append_to_array_elem(name, &pos.to_string(), val)
        } else if d.is_assoc() {
            d.append_to_assoc_elem(name, &pos.to_string(), val)
        } else {
            let data = d.get_as_single()?;
            self.set_uninit_array(layer, name, false)?;
            self.append_elem(layer, name, 0, &data)?;
            self.append_elem(layer, name, pos, val)
        }
    }

    pub(super) fn set_uninit_array(&mut self, layer: usize,
        name: &str, i_flag: bool,
    ) -> Result<(), ExecError> {
        let obj = if i_flag {
            Box::new(Uninit::new("ai")) as Box::<dyn Data>
        }else {
            Box::new(ArrayData::from(Some(vec![]))) as Box::<dyn Data>
        };

        self.set_entry(layer, name, obj)
    }

    pub fn set_entry(&mut self, layer: usize, name: &str,
                     data: Box::<dyn Data>) -> Result<(), ExecError> {
        if self.has_flag(name, 'r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }

        self.params[layer].insert(name.to_string(), data);
        Ok(())
    }

    pub fn remove_entry(&mut self, layer: usize, name: &str) -> Result<bool, ExecError> {
        if self.has_flag(name, 'r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        
        if self.params[layer].contains_key(name) {
            self.params[layer].remove(name);
            if layer == 0 {
                unsafe{env::remove_var(name)};
            }

            return Ok(true);
        }
        Ok(false)
    }
}
