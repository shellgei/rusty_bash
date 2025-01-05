//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;
use super::Data;

#[derive(Debug, Clone, Default)]
pub struct ArrayData {
    body: HashMap<usize, String>,
}

impl From<Vec<String>> for ArrayData {
    fn from(v: Vec<String>) -> Self {
        let mut ans = Self { body: HashMap::new() };

        for i in 0..v.len() {
            ans.body.insert(i, v[i].clone());
        }

        ans
    }
}

impl Data for ArrayData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        let mut formatted = String::new();
        formatted += "(";
        for i in self.keys() {
            formatted += &format!("[{}]=\"{}\" ", i, &self.body[&i]).clone();
        };
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn set_as_array(&mut self, key: &str, value: &str) -> Result<(), String> {
        if let Ok(n) = key.parse::<usize>() {
            self.body.insert(n, value.to_string());
            return Ok(());
        }
        Err("invalid index".to_string())
    }

    fn get_as_array(&mut self, key: &str) -> Option<String> {
        if key == "@" || key == "*" {
            return Some(self.values().join(" "));
        }

        let n = key.parse::<usize>().ok()?;
        self.body.get(&n).cloned()
    }

    fn get_all_as_array(&mut self) -> Option<Vec<String>> {
        Some(self.values().clone())
    }

    fn get_as_single(&mut self) -> Option<String> {
        self.body.get(&0).map(|v| Some(v.clone()))?
    }

    fn is_array(&self) -> bool {true}
    fn len(&mut self) -> usize { self.body.len() }
}

impl ArrayData {
    pub fn values(&self) -> Vec<String> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| e.0.clone()).collect();
        keys.sort();
        keys.iter().map(|i| self.body[i].clone()).collect()
    }

    pub fn keys(&self) -> Vec<usize> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| e.0.clone()).collect();
        keys.sort();
        keys
    }
}
