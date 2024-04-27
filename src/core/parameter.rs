//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::env;

impl ShellCore {
    pub fn get_param_ref(&mut self, key: &str) -> &str {
        if key.len() == 1 {
            let c = key.chars().nth(0).unwrap();
            if '0' <= c && c <= '9' {
                let n = key.parse::<usize>().unwrap();
                if n < self.position_parameters.len() {
                    return &self.position_parameters[n];
                }else{
                    return "";
                }
            }
        }

        if  self.parameters.get(key) == None {
            if let Ok(val) = env::var(key) {
                self.set_param(key, &val);
            }
        }

        match self.parameters.get(key) {
            Some(val) => val,
            None      => "",
        }
    }

    pub fn set_param(&mut self, key: &str, val: &str) {
        match env::var(key) {
            Ok(_) => env::set_var(key, val),
            _     => {},
        }

        self.parameters.insert(key.to_string(), val.to_string());
    }
}
