//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod single;

use crate::core::HashMap;
use self::single::SingleData;

#[derive(Debug, Clone, Default)]
pub enum Value {
    #[default]
    None,
    //Single(String),
    Single(SingleData),
    AssocArray(HashMap::<String, String>),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub value: Value,
    pub attributes: String,
    pub dynamic_get: Option<fn(&mut Variable) -> Value>,
    pub dynamic_set: Option<fn(&mut Variable, &str) -> Value>,
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
            value: Value::Array(vals),
            ..Default::default()
        }
    }
}

impl Variable {
    pub fn get_value(&mut self) -> Value {
        match self.dynamic_get {
            Some(f) => f(self).clone(),
            None    => self.value.clone(),
        }
    }

    pub fn not_set(v: &mut Variable, _var: &str) -> Value {
        v.value.clone()
    }
}
