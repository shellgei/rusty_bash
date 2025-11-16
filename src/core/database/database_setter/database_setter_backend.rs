//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{ArrayData, Data, Uninit};
use crate::core::DataBase;
use crate::error::exec::ExecError;

impl DataBase {
    pub fn set_elem(&mut self, layer: usize, name: &str,
        pos: isize, val: &String, i_flag: bool
        ) -> Result<(), ExecError> {
        if let Some(d) = self.params[layer].get_mut(name) {
            if let Some(init_d) = d.initialize() {
                *d = init_d;
            }

            if d.is_array() {
                d.set_as_array(&pos.to_string(), val)
            } else if d.is_assoc() {
                return d.set_as_assoc(&pos.to_string(), val);
            } else if d.is_single() {
                let data = d.get_as_single()?;
                self.set_new_entry(layer, name, Some(vec![]), i_flag)?;

                if !data.is_empty() {
                    self.set_elem(layer, name, 0, &data, i_flag)?;
                }
                self.set_elem(layer, name, pos, val, i_flag)
            } else {
                self.set_new_entry(layer, name, Some(vec![]), i_flag)?;
                self.set_elem(layer, name, pos, val, i_flag)
            }
        } else {
            self.set_new_entry(layer, name, Some(vec![]), i_flag)?;
            self.set_elem(layer, name, pos, val, i_flag)
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
                d.append_to_array_elem(&pos.to_string(), val)
            } else if d.is_assoc() {
                return d.append_to_assoc_elem(&pos.to_string(), val);
            } else {
                let data = d.get_as_single()?;
                self.set_new_entry(layer, name, Some(vec![]), false)?;
                self.append_elem(layer, name, 0, &data)?;
                self.append_elem(layer, name, pos, val)
            }
        } else {
            self.set_new_entry(layer, name, Some(vec![]), false)?;
            self.set_elem(layer, name, pos, val, false)
        }
    }

    pub fn set_new_entry(&mut self, layer: usize,
        name: &str, v: Option<Vec<String>>, i_flag: bool
    ) -> Result<(), ExecError> {
        let obj = if i_flag {
            Box::new(Uninit::new("ai")) as Box::<dyn Data>
        }else if v.is_none() {
            Box::new(Uninit::new("a")) as Box::<dyn Data>
        }else {
            Box::new(ArrayData::from(v)) as Box::<dyn Data>
        };

        self.params[layer].insert(name.to_string(), obj);
        Ok(())
    }
}
