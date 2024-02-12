//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::ShellCore;

impl ShellCore {
    pub fn get_param_ref(&self, key: &str) -> &str {
        match self.parameters.get(key) {
            Some(val) => val,
            None      => "",
        }
    }

    pub fn set_param(&mut self, key: &str, val: &str) {
        self.parameters.insert(key.to_string(), val.to_string());
    }
}
