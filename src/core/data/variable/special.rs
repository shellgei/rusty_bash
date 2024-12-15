//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::data::Value;
use crate::core::Variable;

#[derive(Debug, Clone)]
pub struct SpecialData {
    pub attributes: String,
    pub data: String,
    pub dynamic_get: fn(&mut Variable) -> Value,
    //pub dynamic_set: Option<fn(&mut Self, &str) -> Value>,
}

impl SpecialData {
    pub fn not_set(v: &mut SpecialData, _var: &str) -> Value {
        Value::Special(v.clone())
    }
}
