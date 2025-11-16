//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

mod database_setter_backend;

use std::env;
use super::{
    ArrayData, AssocData, Data, IntArrayData, IntAssocData, IntData, SingleData, Uninit,
};
use crate::core::DataBase;
use crate::error::exec::ExecError;
use crate::utils::restricted_shell;

impl DataBase {
    fn write_check(&mut self, name: &str, values: &Option<Vec<String>>
    ) -> Result<(), ExecError> {
        Self::name_check(name)?;
        if self.has_flag(name, 'r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        restricted_shell::check(self, name, values)?;
        Ok(())
    }

    pub fn init_as_num(&mut self, name: &str, value: &str, layer: Option<usize>,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![]))?;

        let mut data = IntData::new();

        if !value.is_empty() {
            match value.parse::<isize>() {
                Ok(n) => data.body = n,
                Err(e) => return Err(ExecError::Other(e.to_string())),
            }
        }

        let layer = self.get_target_layer(name, layer);
        self.params[layer].insert(name.to_string(), Box::new(data));
        Ok(())
    }

    pub fn set_param(
        &mut self,
        name: &str,
        val: &str,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![val.to_string()]))?;

        if name == "BASH_ARGV0" {
            let n = layer.unwrap_or(self.get_layer_num() - 1);
            self.position_parameters[n][0] = val.to_string();
        }

        if !self.flags.contains('r') && (self.flags.contains('a') || self.has_flag(name, 'x')) {
            env::set_var(name, "");
        }

        let layer = self.get_target_layer(name, layer);
        let db_layer = &mut self.params[layer];

        if db_layer.get(name).is_none() {
            db_layer.insert(name.to_string(), Box::new(SingleData::from("")));
        }

        let d = db_layer.get_mut(name).unwrap();
        if let Some(init_d) = d.initialize() {
            *d = init_d;
        }

        d.set_as_single(val)?;

        if env::var(name).is_ok() {
            let v = d.get_as_single()?;
            env::set_var(name, &v);
        }
        Ok(())
    }

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
        let db_layer = &mut self.params[layer];

        if db_layer.get(name).is_none() {
            db_layer.insert(name.to_string(), Box::new(SingleData::from("")));
        }

        let d = db_layer.get_mut(name).unwrap();

        if d.is_array() {
            return d.append_to_array_elem("0", val);
        }

        d.append_as_single(val)?;

        if let Ok(name) = env::var(name) {
            let v = d.get_as_single()?;
            env::set_var(name, v);
        }

        Ok(())
    }

    pub fn set_param2(
        &mut self,
        name: &str,
        index: &str,
        val: &str,
        layer: Option<usize>,
    ) -> Result<(), ExecError> {
        if index.is_empty() {
            return self.set_param(name, val, layer);
        }

        if self.is_array(name) {
            if let Ok(n) = index.parse::<isize>() {
                self.set_array_elem(name, val, n, layer, false)?;
            }
        } else if self.is_assoc(name) {
            self.set_assoc_elem(name, index, val, layer)?;
        } else {
            match index.parse::<isize>() {
                Ok(n) => self.set_array_elem(name, val, n, layer, false)?,
                _ => self.set_assoc_elem(name, index, val, layer)?,
            }
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

    pub fn set_array_elem(&mut self, name: &str, val: &str,
        pos: isize, layer: Option<usize>, append: bool,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![val.to_string()]))?;

        let layer = self.get_target_layer(name, layer);
        let i_flag = self.has_flag(name, 'i');
        match append {
            false => self.set_elem(layer, name, pos, &val.to_string(), i_flag),
            true  => self.append_elem(layer, name, pos, &val.to_string()),
        }
    }

    pub fn set_assoc_elem(&mut self, name: &str, key: &str,
        val: &str, layer: Option<usize>,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![val.to_string()]))?;

        let layer = self.get_target_layer(name, layer);
        match self.params[layer].get_mut(name) {
            Some(v) => {
                if let Some(init_v) = v.initialize() {
                    *v = init_v;
                }
                v.set_as_assoc(key, val)
            }
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }

    pub fn append_to_assoc_elem(&mut self, name: &str, key: &str,
        val: &str, layer: Option<usize>,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![val.to_string()]))?;

        let layer = self.get_target_layer(name, layer);
        match self.params[layer].get_mut(name) {
            Some(v) => v.append_to_assoc_elem(key, val),
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }

    pub fn set_array(
        &mut self,
        name: &str,
        v: Option<Vec<String>>,
        layer: Option<usize>,
        i_flag: bool,
    ) -> Result<(), ExecError> {
        self.write_check(name, &v)?;

        let layer = self.get_target_layer(name, layer);
        let db_layer = &mut self.params[layer];

        if i_flag {
            let mut obj = IntArrayData::new();
            if let Some(v) = v {
                for (i, e) in v.into_iter().enumerate() {
                    obj.set_as_array(&i.to_string(), &e)?;
                }
            }
            db_layer.insert(name.to_string(), Box::new(obj));
            return Ok(());
        }

        if v.is_none() {
            db_layer.insert(name.to_string(), Box::new(Uninit::new("a")));
            return Ok(());
        }

        let mut obj = ArrayData::new();
        for (i, e) in v.unwrap().into_iter().enumerate() {
            obj.set_as_array(&i.to_string(), &e)?;
        }
        db_layer.insert(name.to_string(), Box::new(obj));
        Ok(())
    }

    pub fn set_assoc(
        &mut self,
        name: &str,
        layer: Option<usize>,
        set_array: bool,
        i_flag: bool,
    ) -> Result<(), ExecError> {
        self.write_check(name, &None)?;

        let obj = if i_flag {
            Box::new(IntAssocData::new()) as Box::<dyn Data>
        } else if set_array {
            Box::new(AssocData::new()) as Box::<dyn Data>
        } else {
            Box::new(Uninit::new("A")) as Box::<dyn Data>
        };

        let layer = self.get_target_layer(name, layer);
        self.params[layer].insert(name.to_string(), obj);
        Ok(())
    }

    pub fn set_flag(&mut self, name: &str, flag: char, layer: Option<usize>) {
        let layer = match layer {
            None => self.position_parameters.len() - 1,
            Some(lay) => lay,
        };

        let rf = &mut self.params[layer];
        match rf.get_mut(name) {
            Some(d) => { let _ = d.set_flag(flag); },
            None => {
                let obj = match flag {
                    'i' => Box::new(IntData::new()) as Box::<dyn Data>,
                    _ => Box::new(Uninit::new(&flag.to_string())) as Box::<dyn Data>,
                };
                rf.insert(name.to_string(), obj);
            }
        }
    }

    pub fn unset_flag(&mut self, name: &str, flag: char, layer: Option<usize>) {
        let layer = match layer {
            None => self.position_parameters.len() - 1,
            Some(lay) => lay,
        };

        let rf = &mut self.params[layer];
        if let Some(d) = rf.get_mut(name) {
            let _ = d.unset_flag(flag);
        }
    }
}
