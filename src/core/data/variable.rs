//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::HashMap;
use crate::elements::array::Array;
use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub enum Value {
    #[default]
    None,
    Single(Word),
    EvaluatedSingle(String),
    Array(Array),
    AssocArray(HashMap::<String, String>),
    EvaluatedArray(Vec<String>),
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
            value: Value::EvaluatedSingle(s.to_string()),
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
            value: Value::EvaluatedArray(vals),
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

    /*
    pub fn new_assoc() -> Self {
        Variable {
            value: Value::AssocArray(HashMap::new()),
            ..Default::default()
        }
    }*/
}
