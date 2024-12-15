//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AssocData {
    pub data: HashMap<String, String>,
}

/*
impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self {
            data: s.to_string(),
            ..Default::default()
        }
    }
}

impl From<&String> for SingleData {
    fn from(s: &String) -> Self {
        Self {
            data: s.clone(),
            ..Default::default()
        }
    }
}
*/

impl From<HashMap<String, String>> for AssocData {
    fn from(hm: HashMap<String, String>) -> Self {
        Self {
            data: hm,
            ..Default::default()
        }
    }
}
