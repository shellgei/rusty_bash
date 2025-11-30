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
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![val.to_string()]))?;

        if name == "BASH_ARGV0" {
            let n = layer.unwrap_or(self.get_layer_num() - 1);
            self.position_parameters[n][0] += val;
        }

        if !self.flags.contains('r')
            && (self.flags.contains('a') || self.has_flag(name, 'x'))
            && env::var(name).is_err()
        {
            env::set_var(name, "");
        }

        let layer = self.get_target_layer(name, layer);

        if self.params[layer].get(name).is_none() {
            self.set_entry(layer, name, Box::new(SingleData::from("")))?;
        }

        let d = self.params[layer].get_mut(name).unwrap();
        if d.is_array() {
            return d.append_to_array_elem(name, "0", val);
        }

        d.append_as_single(name, val)?;

        if env::var(name).is_ok() {
            let v = d.get_as_single()?;
            env::set_var(name, v);
        }

        Ok(())
    }

    pub fn append_param2(
        &mut self,
        name: &str,
        index: &str,
        val: &str,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.append_param(name, val, layer);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<isize>() {
                self.set_array_elem(name, val, n, layer, true)?;
            }
        } else if self.is_assoc(name) {
            self.append_to_assoc_elem(name, index, val, layer)?;
        } else {
            match index.parse::<isize>() {
                Ok(n) => self.set_array_elem(name, val, n, layer, true)?,
                _ => self.append_to_assoc_elem(name, index, val, layer)?,
            }
        }
        Ok(())
    }

    pub fn append_to_assoc_elem(&mut self, name: &str, key: &str,
        val: &str, layer: Option<usize>,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![val.to_string()]))?;

        let layer = self.get_target_layer(name, layer);
        match self.params[layer].get_mut(name) {
            Some(v) => v.append_to_assoc_elem(name, key, val),
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }
}
