//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::Variable;
use crate::core::data::Value;

#[derive(Debug, Clone, Default)]
pub struct SingleData {
    pub data: String,
    pub attributes: String,
    pub dynamic_get: Option<fn(&mut Variable) -> Value>,
    pub dynamic_set: Option<fn(&mut Variable, &str) -> Value>,
}

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

impl From<String> for SingleData {
    fn from(s: String) -> Self {
        Self {
            data: s,
            ..Default::default()
        }
    }
}
