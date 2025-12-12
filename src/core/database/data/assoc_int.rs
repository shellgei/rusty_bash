//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::assoc::AssocData;
use super::Data;
use crate::error::exec::ExecError;
use crate::utils;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct IntAssocData {
    body: HashMap<String, isize>,
    last: Option<String>,
    pub flags: String,
}

impl Data for IntAssocData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn get_fmt_string(&mut self) -> String {
        self._get_fmt_string()
    }

    fn _get_fmt_string(&self) -> String {
        let mut formatted = String::new();
        formatted += "(";
        for k in self.keys() {
            let v = &self.get(&k).unwrap_or("".to_string());
            let mut ansi = utils::to_ansi_c(v);
            if ansi == *v {
                ansi = format!("\"{}\"", &ansi);
            }

            let k = utils::to_ansi_c(&k);
            formatted += &format!("[{}]={} ", k, &ansi);
        }

        formatted += ")";
        formatted
    }

    fn clear(&mut self) {
        self.body.clear();
    }

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        let n = super::to_int(value)?;
        self.body.insert("0".to_string(), n);
        Ok(())
    }

    fn set_as_assoc(&mut self, name: &str, key: &str,
                    value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        let n = super::to_int(value)?;
        self.body.insert(key.to_string(), n);
        self.last = Some(value.to_string());
        Ok(())
    }

    fn append_to_assoc_elem(&mut self, name: &str, key: &str,
                            value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        let n = super::to_int(value)?;

        if let Some(v) = self.body.get(key) {
            self.body.insert(key.to_string(), v + n);
        } else {
            self.body.insert(key.to_string(), n);
        }
        self.last = Some(value.to_string());
        Ok(())
    }

    fn get_as_assoc(&mut self, key: &str) -> Result<String, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.values().join(" "));
        }

        match self.body.get(key) {
            Some(s) => Ok(s.to_string()),
            None => Err(ExecError::ArrayIndexInvalid(key.to_string())),
        }
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        if let Some(s) = self.body.get("0") {
            return Ok(s.to_string());
        }

        self.last
            .clone()
            .ok_or(ExecError::Other("No last input".to_string()))
    }

    fn get_str_type(&self) -> Box<dyn Data> {
        let mut hash = HashMap::new();
        for d in &self.body {
            hash.insert(d.0.to_string(), d.1.to_string());
        }

        let mut new_d = AssocData::from(hash);
        new_d.flags = self.flags.clone();
        new_d.unset_flag('i');
        Box::new(new_d)
    }

    fn is_assoc(&self) -> bool {
        true
    }
    fn len(&mut self) -> usize {
        self.body.len()
    }

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }
        Ok(self.body.contains_key(key))
    }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let s = *self.body.get(key).unwrap_or(&0);

        Ok(s.to_string().len())
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(self.keys().clone())
    }

    fn get_all_as_array(&mut self, skip_none: bool) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }

        let mut keys = self.keys();
        keys.sort();
        let mut ans = vec![];
        for i in keys {
            match self.body.get(&i) {
                Some(s) => ans.push(s.to_string()),
                None => {
                    if !skip_none {
                        ans.push("".to_string());
                    }
                }
            }
        }
        Ok(ans)
    }

    fn get_vec_from(&mut self, _: usize, skip_non: bool) -> Result<Vec<String>, ExecError> {
        self.get_all_as_array(skip_non)
    }

    fn remove_elem(&mut self, key: &str) -> Result<(), ExecError> {
        if key == "*" || key == "@" {
            //     self.body.clear();
            return Ok(());
        }

        self.body.remove(key);
        Ok(())
    }

    fn set_flag(&mut self, flag: char) {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
    }

    fn unset_flag(&mut self, flag: char) {
        self.flags.retain(|e| e != flag);
    }

    fn has_flag(&mut self, flag: char) -> bool {
        if flag == 'i' {
            return true;
        }
        self.flags.contains(flag)
    }
}

impl IntAssocData {
    pub fn new() -> Self {
        Self { body: HashMap::new(), last: None, flags: "i".to_string() }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        Some(self.body.get(key).unwrap_or(&0).to_string())
    }

    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.body.iter().map(|e| e.0.clone()).collect();
        keys.sort();
        keys
        //self.body.iter().map(|e| e.0.clone()).collect()
    }

    pub fn values(&self) -> Vec<String> {
        self.body.iter().map(|e| e.1.to_string()).collect()
    }
}
