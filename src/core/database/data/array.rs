//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::array_uninit::UninitArray;
use super::Data;
use crate::error::exec::ExecError;
use crate::utils;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ArrayData {
    body: HashMap<usize, String>,
    pub flags: String,
}

impl From<HashMap<usize, String>> for ArrayData {
    fn from(h: HashMap<usize, String>) -> Self {
        Self { body: h, flags: String::new() }
    }
}

impl From<Option<Vec<String>>> for ArrayData {
    fn from(v: Option<Vec<String>>) -> Self {
        let mut ans = Self::default();
        v.unwrap().into_iter().enumerate().for_each(|(i, e)| {
            ans.body.insert(i, e);
        });
        ans
    }
}

impl Data for ArrayData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        let mut formatted = "(".to_string();
        for i in self.keys() {
            let ansi = utils::to_ansi_c(&self.body[&i]);
            if ansi == self.body[&i] {
                formatted += &format!("[{}]=\"{}\" ", i, &ansi.replace("$", "\\$"));
            } else {
                formatted += &format!("[{}]={} ", i, &ansi);
            }
        }
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn clear(&mut self) {
        self.body.clear();
    }
    fn is_initialized(&self) -> bool {
        true
    }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body.insert(0, value.to_string());
        Ok(())
    }

    fn append_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        if let Some(v) = self.body.get(&0) {
            self.body.insert(0, v.to_owned() + value);
        } else {
            self.body.insert(0, value.to_string());
        }
        Ok(())
    }

    fn set_as_array(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        let n = self.index_of(key)?;
        self.body.insert(n, value.to_string());
        Ok(())
    }

    fn append_to_array_elem(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        let n = self.index_of(key)?;
        if let Some(v) = self.body.get(&n) {
            self.body.insert(n, v.to_owned() + value);
        } else {
            self.body.insert(n, value.to_string());
        }
        Ok(())
    }

    fn get_as_array(&mut self, key: &str, ifs: &str) -> Result<String, ExecError> {
        if key == "@" {
            return Ok(self.values().join(" "));
        }
        if key == "*" {
            return Ok(self.values().join(ifs));
        }

        let n = self.index_of(key)?;
        Ok(self.body.get(&n).unwrap_or(&"".to_string()).clone())
    }

    fn get_vec_from(&mut self, pos: usize, skip_non: bool) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }

        let keys = self.keys();
        let max = *keys.iter().max().unwrap();
        let mut ans = vec![];
        for i in pos..(max + 1) {
            match self.body.get(&i) {
                Some(s) => ans.push(s.clone()),
                None => {
                    if !skip_non {
                        ans.push("".to_string());
                    }
                }
            }
        }
        Ok(ans)
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(self.keys().iter().map(|k| k.to_string()).collect())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        self.body
            .get(&0)
            .map(|v| Ok(v.clone()))
            .ok_or(ExecError::Other("No entry".to_string()))?
    }

    fn is_array(&self) -> bool {
        true
    }
    fn len(&mut self) -> usize {
        self.body.len()
    }

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }
        let n = self.index_of(key)?;
        Ok(self.body.contains_key(&n))
    }

    fn index_based_len(&mut self) -> usize {
        match self.body.iter().map(|e| e.0).max() {
            Some(n) => *n + 1,
            None => 0,
        }
    }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let n = self.index_of(key)?;
        let s = self.body.get(&n).unwrap_or(&"".to_string()).clone();

        Ok(s.chars().count())
    }

    fn remove_elem(&mut self, key: &str) -> Result<(), ExecError> {
        if key == "*" || key == "@" {
            self.body.clear();
            return Ok(());
        }

        let index = self.index_of(key)?;
        self.body.remove(&index);
        Ok(())
    }

    fn set_flag(&mut self, flag: char) -> Result<(), ExecError> {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
        Ok(())
    }

    fn unset_flag(&mut self, flag: char) -> Result<(), ExecError> {
        self.flags.retain(|e| e != flag);
        Ok(())
    }

    fn has_flag(&mut self, flag: char) -> Result<bool, ExecError> {
        Ok(self.flags.contains(flag))
    }
}

impl ArrayData {
    pub fn set_new_entry(
        db_layer: &mut HashMap<String, Box<dyn Data>>,
        name: &str,
        v: Option<Vec<String>>,
    ) -> Result<(), ExecError> {
        if v.is_none() {
            db_layer.insert(name.to_string(), UninitArray {flags: String::new()}.boxed_clone());
        } else {
            db_layer.insert(name.to_string(), Box::new(ArrayData::from(v)));
        }
        Ok(())
    }

    pub fn set_elem(
        db_layer: &mut HashMap<String, Box<dyn Data>>,
        name: &str,
        pos: isize,
        val: &String,
    ) -> Result<(), ExecError> {
        if let Some(d) = db_layer.get_mut(name) {
            if d.is_array() {
                if !d.is_initialized() {
                    let flags = d.give_inherit_flags();
                    *d = ArrayData {body: HashMap::new(), flags: flags}.boxed_clone();
                }

                d.set_as_array(&pos.to_string(), val)
            } else if d.is_assoc() {
                return d.set_as_assoc(&pos.to_string(), val);
            } else if d.is_single() {
                let data = d.get_as_single()?;
                ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;

                if !data.is_empty() {
                    Self::set_elem(db_layer, name, 0, &data)?;
                }
                Self::set_elem(db_layer, name, pos, val)
            } else {
                ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
                Self::set_elem(db_layer, name, pos, val)
            }
        } else {
            ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
            Self::set_elem(db_layer, name, pos, val)
        }
    }

    pub fn append_elem(
        db_layer: &mut HashMap<String, Box<dyn Data>>,
        name: &str,
        pos: isize,
        val: &String,
    ) -> Result<(), ExecError> {
        if let Some(d) = db_layer.get_mut(name) {
            if d.is_array() {
                if !d.is_initialized() {
                    let flags = d.give_inherit_flags();
                    *d = ArrayData {body: HashMap::new(), flags: flags}.boxed_clone();
                }

                d.append_to_array_elem(&pos.to_string(), val)
            } else if d.is_assoc() {
                return d.append_to_assoc_elem(&pos.to_string(), val);
            } else {
                let data = d.get_as_single()?;
                ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
                Self::append_elem(db_layer, name, 0, &data)?;
                Self::append_elem(db_layer, name, pos, val)
            }
        } else {
            ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
            Self::set_elem(db_layer, name, pos, val)
        }
    }

    pub fn values(&self) -> Vec<String> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| *e.0).collect();
        keys.sort();
        keys.iter().map(|i| self.body[i].clone()).collect()
    }

    pub fn keys(&self) -> Vec<usize> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| *e.0).collect();
        keys.sort();
        keys
    }

    fn index_of(&mut self, key: &str) -> Result<usize, ExecError> {
        let mut index = match key.parse::<isize>() {
            Ok(i) => i,
            _ => return Err(ExecError::ArrayIndexInvalid(key.to_string())),
        };

        if index >= 0 {
            return Ok(index as usize);
        }

        let keys = self.keys();
        let max = match keys.iter().max() {
            Some(n) => *n as isize,
            None => -1,
        };
        index += max + 1;

        if index < 0 {
            return Err(ExecError::ArrayIndexInvalid(key.to_string()));
        }

        Ok(index as usize)
    }
}
