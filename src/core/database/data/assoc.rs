//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AssocData {
    body: HashMap<String, String>,
}

impl From<HashMap<String, String>> for AssocData {
    fn from(hm: HashMap<String, String>) -> Self {
        Self { body: hm }
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

    fn set_as_assoc(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.body.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn get_as_assoc(&mut self, key: &str) -> Option<String> {
        if key == "@" || key == "*" {
            return Some(self.values().join(" "));
        }

        match self.body.get(key) {
            Some(s) => Some(s.to_string()),
            None => None,
        }
    }

    fn is_assoc(&self) -> bool {true}
    fn len(&mut self) -> usize { self.body.len() }
}

impl AssocData {
    pub fn get(&self, key: &str) -> Option<String> {
        self.body.get(key).cloned()
    }

    pub fn keys(&self) -> Vec<String> {
        self.body.iter().map(|e| e.0.clone()).collect()
    }

    pub fn values(&self) -> Vec<String> {
        self.body.iter().map(|e| e.1.clone()).collect()
    }

    /*
    pub fn set(&mut self, key: &String, val: &String) -> bool {
        self.body.insert(key.to_string(), val.to_string());
        true
    }

    pub fn print(&self, k: &str) {
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
        println!("{}={}", k.to_string(), formatted); 
    }
    */
}
