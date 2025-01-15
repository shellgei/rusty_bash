//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use super::Data;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AssocData {
    body: HashMap<String, String>,
    last: Option<String>,
}

impl From<HashMap<String, String>> for AssocData {
    fn from(hm: HashMap<String, String>) -> Self {
        Self { body: hm, last: None, }
    }
}

impl Data for AssocData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        let mut formatted = String::new();
        formatted += "(";
        for k in self.keys() {
            let v = self.get(&k).unwrap_or("".to_string());
            formatted += &format!("[{}]=\"{}\" ", k, v);
        }
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn set_as_assoc(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        self.body.insert(key.to_string(), value.to_string());
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

    fn get_as_single(&mut self) -> Result<String, ExecError> { self.last.clone().ok_or(ExecError::Other("No last input".to_string())) }

    fn is_assoc(&self) -> bool {true}
    fn len(&mut self) -> usize { self.body.len() }
}

impl AssocData {
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str) -> Result<(), ExecError> {
        db_layer.insert(name.to_string(), Box::new(AssocData::default()));
        Ok(())
    }

    pub fn set_elem(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str,
                     key: &String, val: &String) -> Result<(), ExecError> {
        match db_layer.get_mut(name) {
            Some(v) => v.set_as_assoc(key, val), 
            _ => Err(ExecError::Other("TODO".to_string())),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.body.get(key).cloned()
    }

    pub fn keys(&self) -> Vec<String> {
        self.body.iter().map(|e| e.0.clone()).collect()
    }

    pub fn values(&self) -> Vec<String> {
        self.body.iter().map(|e| e.1.clone()).collect()
    }
}
