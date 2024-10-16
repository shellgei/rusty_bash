//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug)]
pub struct Data {
    pub flags: String,
    pub parameters: HashMap<String, String>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            flags: String::new(),
            parameters: HashMap::new(),
        }
    }

    pub fn get_param(&self, key: &str) -> String {
        match self.parameters.get(key) {
            Some(val) => val,
            None      => "",
        }.to_string()
    }

    pub fn set_param(&mut self, key: &str, val: &str) {
        self.parameters.insert(key.to_string(), val.to_string());
    }
}
