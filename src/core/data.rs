//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;
use std::env;

#[derive(Debug, Default)]
pub struct Data {
    pub flags: String,
    pub parameters: HashMap<String, String>,
}

impl Data {
    pub fn get_param(&mut self, key: &str) -> String {
        if ! self.parameters.contains_key(key) {
            if let Ok(val) = env::var(key) {
                self.set_param(key, &val);
            }
        }

        match self.parameters.get(key) {
            Some(val) => val,
            None      => "",
        }.to_string()
    }

    pub fn set_param(&mut self, key: &str, val: &str) {
        self.parameters.insert(key.to_string(), val.to_string());
    }
}
