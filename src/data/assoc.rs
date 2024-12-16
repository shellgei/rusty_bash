//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AssocData {
    data: HashMap<String, String>,
}

impl From<HashMap<String, String>> for AssocData {
    fn from(hm: HashMap<String, String>) -> Self {
        Self { data: hm }
    }
}

impl AssocData {
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    pub fn keys(&self) -> Vec<String> {
        self.data.iter().map(|e| e.0.clone()).collect()
    }

    pub fn values(&self) -> Vec<String> {
        self.data.iter().map(|e| e.1.clone()).collect()
    }

    pub fn set(&mut self, key: String, val: String) -> bool {
        self.data.insert(key, val);
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
}
