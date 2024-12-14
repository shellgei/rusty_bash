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

