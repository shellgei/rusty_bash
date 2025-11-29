//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{ArrayData, Data, Uninit};
use crate::core::DataBase;
use crate::error::exec::ExecError;

impl DataBase {
    pub fn set_elem(&mut self, layer: usize, name: &str,
        pos: isize, val: &String, flags: &str
        ) -> Result<(), ExecError> {
        if let Some(d) = self.params[layer].get_mut(name) {
            if let Some(init_d) = d.initialize() {
                *d = init_d;
            }

            if d.is_array() {
                d.set_as_array(name, &pos.to_string(), val)
            } else if d.is_assoc() {
                d.set_as_assoc(name, &pos.to_string(), val)
            } else if d.is_single() {
                let data = d.get_as_single()?;
                self.set_uninit_array(layer, name, Some(vec![]), &flags)?;

                if !data.is_empty() {
                    self.set_elem(layer, name, 0, &data, flags)?;
                }
                self.set_elem(layer, name, pos, val, flags)
            } else {
                self.set_uninit_array(layer, name, Some(vec![]), &flags)?;
                self.set_elem(layer, name, pos, val, flags)
            }
        } else {
            self.set_uninit_array(layer, name, Some(vec![]), &flags)?;
            self.set_elem(layer, name, pos, val, flags)
        }
    }

    pub fn append_elem(&mut self, layer: usize, 
        name: &str, pos: isize, val: &String,
    ) -> Result<(), ExecError> {
        if let Some(d) = self.params[layer].get_mut(name) {
            if let Some(init_d) = d.initialize() {
                *d = init_d;
            }

            if d.is_array() {
                d.append_to_array_elem(name, &pos.to_string(), val)
            } else if d.is_assoc() {
                d.append_to_assoc_elem(name, &pos.to_string(), val)
            } else {
                let data = d.get_as_single()?;
                self.set_uninit_array(layer, name, Some(vec![]), "")?;
                self.append_elem(layer, name, 0, &data)?;
                self.append_elem(layer, name, pos, val)
            }
        } else {
            self.set_uninit_array(layer, name, Some(vec![]), "")?;
            self.set_elem(layer, name, pos, val, "")
        }
    }

    pub fn set_uninit_array(&mut self, layer: usize,
        name: &str, v: Option<Vec<String>>, flags: &str,
    ) -> Result<(), ExecError> {
        let obj = if flags.contains('i') {
            Box::new(Uninit::new("ai")) as Box::<dyn Data>
        }else if v.is_none() {
            Box::new(Uninit::new("a")) as Box::<dyn Data>
        }else {
            Box::new(ArrayData::from(v)) as Box::<dyn Data>
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

    pub fn remove_entry(&mut self, layer: usize, name: &str) -> Result<(), ExecError> {
        if self.has_flag(name, 'r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }

        if self.params[layer].contains_key(name) {
            self.params[layer].remove(name);
        }
        Ok(())
    }
}
