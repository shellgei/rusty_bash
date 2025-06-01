//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils;
use crate::error::exec::ExecError;
use super::Data;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct IntAssocData {
    body: HashMap<String, isize>,
    last: Option<String>,
}

/*
impl From<HashMap<String, String>> for IntAssocData {
    fn from(hm: HashMap<String, String>) -> Self {
        Self { body: hm, last: None, }
    }
}*/

impl Data for IntAssocData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        let mut formatted = String::new();
        formatted += "(";
        for k in self.keys() {
            let v = &self.get(&k).unwrap_or("".to_string());
            let ansi = utils::to_ansi_c(v);
            if ansi == *v {
                formatted += &format!("[{}]=\"{}\" ", k, &ansi);
            }else{
                formatted += &format!("[{}]={} ", k, &ansi);
            }
        }

        formatted += ")";
        formatted
    }

    fn clear(&mut self) { self.body.clear(); }

    fn set_as_assoc(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        let n = super::to_int(value)?;
        self.body.insert(key.to_string(), n);
        self.last = Some(value.to_string());
        Ok(())
    }

    fn append_to_assoc_elem(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        let n = super::to_int(value)?;

        if let Some(v) = self.body.get(key) {
            self.body.insert(key.to_string(), v + n);
        }else{
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

        self.last.clone().ok_or(ExecError::Other("No last input".to_string()))
    }

    fn is_assoc(&self) -> bool {true}
    fn len(&mut self) -> usize { self.body.len() }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let s = self.body.get(key).unwrap_or(&0).clone();

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
                None => if ! skip_none {
                    ans.push("".to_string());
                },
            }
        }
        Ok(ans)
    }

    fn get_vec_from(&mut self, _: usize, skip_non: bool) -> Result<Vec<String>, ExecError> {
        self.get_all_as_array(skip_non)
    }

    fn remove_elem(&mut self, key: &str) -> Result<(), ExecError> {
        if key == "*" || key == "@" {
            self.body.clear();
            return Ok(());
        }

        self.body.remove(key);
        return Ok(());
    }
}

impl IntAssocData {
    /*
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str) -> Result<(), ExecError> {
        db_layer.insert(name.to_string(), Box::new(IntAssocData::default()));
        Ok(())
    }

    pub fn set_elem(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str,
                     key: &String, val: &String) -> Result<(), ExecError> {
        match db_layer.get_mut(name) {
            Some(v) => v.set_as_assoc(key, val), 
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }

    pub fn append_elem(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str,
                     key: &String, val: &String) -> Result<(), ExecError> {
        match db_layer.get_mut(name) {
            Some(v) => v.append_to_assoc_elem(key, val), 
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }*/

    pub fn get(&self, key: &str) -> Option<String> {
        Some(self.body.get(key).unwrap_or(&0).to_string())
    }

    pub fn keys(&self) -> Vec<String> {
        self.body.iter().map(|e| e.0.clone()).collect()
    }

    pub fn values(&self) -> Vec<String> {
        self.body.iter().map(|e| e.1.to_string()).collect()
    }
}
