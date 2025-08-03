//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::data::Data;
use super::DataBase;
use crate::error::exec::ExecError;
use std::collections::HashSet;
use std::env;

impl DataBase {
    pub fn get_ref(&mut self, name: &str) -> Option<&mut Box<dyn Data>> {
        let num = self.params.len();
        for layer in (0..num).rev() {
            if self.params[layer].get_mut(name).is_some() {
                return self.params[layer].get_mut(name);
            }
        }
        None
    }

    pub fn get_ifs_head(&mut self) -> String {
        let ifs = self.get_param("IFS").unwrap_or(" ".to_string());
        match ifs.as_str() {
            "" => "".to_string(),
            s => s.chars().next().unwrap().to_string(),
        }
    }

    pub fn get_layer_num(&mut self) -> usize {
        self.params.len()
    }

    pub fn get_keys(&mut self) -> Vec<String> {
        let mut keys = HashSet::new();
        for layer in &self.params {
            layer.keys().for_each(|k| {
                keys.insert(k);
            });
        }
        for f in &self.functions {
            keys.insert(f.0);
        }
        let mut ans: Vec<String> = keys.iter().map(|c| c.to_string()).collect();
        ans.sort();
        ans
    }

    pub fn get_layer_pos(&mut self, name: &str) -> Option<usize> {
        let num = self.params.len();
        (0..num)
            .rev()
            .find(|&layer| self.params[layer].contains_key(name))
    }

    pub fn get_position_params(&self) -> Vec<String> {
        match self.position_parameters.last() {
            Some(v) => v[1..].to_vec(),
            _ => vec![],
        }
    }

    pub fn get_indexes_all(&mut self, name: &str) -> Vec<String> {
        let layer = self.position_parameters.len() - 1;
        if name == "@" {
            return self.position_parameters[layer].clone();
        }

        self.get_ref(name)
            .map(|d| d.get_all_indexes_as_array().unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_vec_from(
        &mut self,
        name: &str,
        pos: usize,
        flatten: bool,
    ) -> Result<Vec<String>, ExecError> {
        let layer = self.position_parameters.len() - 1;
        if name == "@" {
            return Ok(self.position_parameters[layer].clone());
        }

        match self.get_ref(name) {
            Some(d) => {
                if let Ok(v) = d.get_vec_from(pos, flatten) {
                    return Ok(v);
                }
                Ok(vec![])
            }
            None => {
                if self.flags.contains('u') {
                    return Err(ExecError::UnboundVariable(name.to_string()));
                }
                Ok(vec![])
            }
        }
    }

    pub fn len(&mut self, name: &str) -> usize {
        if let Some(d) = self.get_ref(name) {
            return d.len();
        }
        0
    }

    pub fn index_based_len(&mut self, name: &str) -> usize {
        if let Some(d) = self.get_ref(name) {
            return d.index_based_len();
        }
        0
    }

    pub fn get_vec(&mut self, name: &str, flatten: bool) -> Result<Vec<String>, ExecError> {
        self.get_vec_from(name, 0, flatten)
    }

    pub fn get_elem(&mut self, name: &str, pos: &str) -> Result<String, ExecError> {
        Self::name_check(name)?;

        let layer = self.get_layer_pos(name);

        if layer.is_none() {
            return match self.flags.contains('u') {
                true => Err(ExecError::UnboundVariable(name.to_string())),
                false => Ok("".to_string()),
            };
        }

        let ifs = self.get_ifs_head();
        self.params[layer.unwrap()]
            .get_mut(name)
            .unwrap()
            .get_as_array_or_assoc(pos, &ifs)
    }

    pub fn get_elem_or_param(&mut self, name: &str, index: &str) -> Result<String, ExecError> {
        match index.is_empty() {
            true => self.get_param(name),
            false => self.get_elem(name, index),
        }
    }

    pub fn get_elem_len(&mut self, name: &str, key: &str) -> Result<usize, ExecError> {
        Self::name_check(name)?;

        if let Some(v) = self.get_ref(name) {
            return v.elem_len(key);
        }

        if self.flags.contains('u') {
            return Err(ExecError::UnboundVariable(name.to_string()));
        }

        Ok(0)
    }

    pub fn get_len(&mut self, name: &str) -> Result<usize, ExecError> {
        Self::name_check(name)?;

        if name == "@" || name == "*" {
            let layer = self.position_parameters.len();
            return Ok(self.position_parameters[layer - 1].len() - 1);
        }

        if let Ok(n) = name.parse::<usize>() {
            return Ok(position_param(self, n)?.chars().count());
        }

        if let Some(v) = self.get_ref(name) {
            return v.elem_len("0");
        }

        if let Ok(v) = env::var(name) {
            return Ok(v.chars().count());
        }

        if self.flags.contains('u') {
            return Err(ExecError::UnboundVariable(name.to_string()));
        }

        Ok(0)
    }

    pub fn get_param(&mut self, name: &str) -> Result<String, ExecError> {
        Self::name_check(name)?;

        if let Some(val) = special_param(self, name) {
            return Ok(val);
        }

        if name == "@" || name == "*" {
            //return connected position params
            return connected_position_params(self, name == "*");
        } //in double quoted subword, this method should not be used

        if let Ok(n) = name.parse::<usize>() {
            return position_param(self, n);
        }

        if let Some(v) = self.get_ref(name) {
            if v.is_special() {
                return v.get_as_single();
            } else if v.is_single_num() {
                let val = v.get_as_single_num()?; //.unwrap_or_default();
                return Ok(val.to_string());
            } else {
                let val = v.get_as_single().unwrap_or_default();
                return Ok(val);
            }
        }

        if let Ok(v) = env::var(name) {
            let _ = self.set_param(name, &v, Some(0));
            return Ok(v);
        }

        if self.flags.contains('u') {
            return Err(ExecError::UnboundVariable(name.to_string()));
        }

        Ok("".to_string())
    }
}

pub fn special_param(db: &DataBase, name: &str) -> Option<String> {
    let val = match name {
        "-" => db.flags.clone(),
        "?" => db.exit_status.to_string(),
        "_" => db.last_arg.clone(),
        "#" => {
            let pos = db.position_parameters.len() - 1;
            (db.position_parameters[pos].len() - 1).to_string()
        }
        _ => return None,
    };

    Some(val)
}

pub fn connected_position_params(db: &mut DataBase, aster: bool) -> Result<String, ExecError> {
    let mut joint = " ".to_string();
    if aster {
        joint = db.get_ifs_head();
    }

    match db.position_parameters.last() {
        Some(a) => Ok(a[1..].join(&joint)),
        _ => Ok("".to_string()),
    }
}

pub fn position_param(db: &DataBase, pos: usize) -> Result<String, ExecError> {
    let layer = db.position_parameters.len();
    match db.position_parameters[layer - 1].len() > pos {
        true => Ok(db.position_parameters[layer - 1][pos].to_string()),
        false => Ok(String::new()),
    }
}

/*
pub fn array_elem(db: &mut DataBase, name: &str, pos: &str) -> Result<String, ExecError> {
    let layer = match db.get_layer_pos(name) {
        Some(n) => n,
        _ => return Ok("".to_string()),
    };

    let ifs = db.get_ifs_head();
    db.params[layer].get_mut(name).unwrap().get_as_array_or_assoc(pos, &ifs)
}*/
