//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data2;

#[derive(Debug, Clone, Default)]
pub struct ArrayData2 {
    body: Vec<String>,
}

impl From<Vec<String>> for ArrayData2 {
    fn from(v: Vec<String>) -> Self {
        Self { body: v }
    }
}

impl Data2 for ArrayData2 {
    fn boxed_clone(&self) -> Box<dyn Data2> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        let mut formatted = String::new();
        formatted += "(";
        for i in 0..self.body.len() {
            formatted += &format!("[{}]=\"{}\" ", i, &self.body[i]).clone();
        };
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn set_as_array(&mut self, key: &str, value: &str) -> bool {
        if let Ok(n) = key.parse::<usize>() {
            if n < self.body.len() {
                self.body[n] = value.to_string();
                return true;
            }
        }
        false
    }

    fn get_as_array(&mut self, key: &str) -> Option<String> {
        if key == "@" || key == "*" {
            return Some(self.body.join(" "));
        }

        match key.parse::<usize>() {
            Ok(n) => {
                match n < self.body.len() {
                    true  => Some(self.body[n].clone()),
                    false => None,
                }
            },
            _ => None
        }
    }

    fn get_all_as_array(&mut self) -> Option<Vec<String>> {
        Some(self.body.clone())
    }

    fn is_array(&self) -> bool {true}
    fn len(&mut self) -> usize { self.body.len() }
}

impl ArrayData2 {
    /*
    pub fn get(&self, key: usize) -> Option<String> {
        match key < self.body.len() {
            true  => Some(self.body[key].clone()),
            false => None,
        }
    }

    pub fn values(&self) -> Vec<String> {
        self.body.clone()
    }*/
}
