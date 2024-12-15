//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod single;
pub mod special;

use crate::core::HashMap;
use self::array::ArrayData;
use self::single::SingleData;
use self::special::SpecialData;

#[derive(Debug, Clone, Default)]
pub enum Value {
    #[default]
    None,
    Special(SpecialData),
    Single(SingleData),
    AssocArray(HashMap::<String, String>),
    Array(ArrayData),
}

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub value: Value,
    pub attributes: String,
}

impl From<&str> for Variable {
    fn from(s: &str) -> Self {
        Variable {
            value: Value::Single(SingleData::from(s)),
            ..Default::default()
        }
    }
}

impl From<HashMap<String, String>> for Variable {
    fn from(hm: HashMap<String, String>) -> Self {
        Variable {
            value: Value::AssocArray(hm),
            ..Default::default()
        }
    }
}

impl From<Vec<String>> for Variable {
    fn from(vals: Vec<String>) -> Self {
        Variable {
            value: Value::Array(ArrayData::from(vals)),
            ..Default::default()
        }
    }
}

impl Variable {
    pub fn set_data(&mut self, data: String) {
        match &mut self.value {
            Value::Single(s) => s.data = data,
            Value::Special(s) => s.data = data,
            _ => {},
        }
    }

    pub fn get_value(&mut self) -> Value {
        match &self.value {
            Value::Special(d) => (d.dynamic_get)(self).clone(),
            _ => self.value.clone(),
        }
    }

    pub fn not_set(v: &mut Variable, _var: &str) -> Value {
        v.value.clone()
    }
}
