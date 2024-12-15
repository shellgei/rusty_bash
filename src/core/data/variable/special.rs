//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::data::Value;
use crate::core::Variable;
use super::single::SingleData;

#[derive(Debug, Clone)]
pub struct SpecialData {
    pub attributes: String,
    pub data: String,
    pub dynamic_get: fn(&mut Variable) -> Value,
    pub dynamic_set: Option<fn(&mut Variable, &str)>,
}

impl SpecialData {
    pub fn not_set(v: &mut SpecialData, _var: &str) -> Value {
        Value::Special(v.clone())
    }

    pub fn get_data(&self, v: &mut Variable, val: &str) {
        match self.dynamic_set {
            Some(f) => f(v, val),
            None    => {},
        }
    }
}
