//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod assoc;
pub mod single;
pub mod special;

use std::collections::HashMap;
use self::array::ArrayData;
use self::assoc::AssocData;
use self::single::SingleData;
use self::special::SpecialData;

#[derive(Debug, Clone, Default)]
pub enum DataType {
    #[default]
    None,
    Special(SpecialData),
    Single(SingleData),
    AssocArray(AssocData),
    Array(ArrayData),
}

#[derive(Debug, Clone, Default)]
pub struct Data {
    pub value: DataType,
    pub attributes: String,
}

impl From<DataType> for Data {
    fn from(v: DataType) -> Self {
        Data {
            value: v,
            ..Default::default()
        }
    }
}

impl From<&str> for Data {
    fn from(s: &str) -> Self {
        Data {
            value: DataType::Single(SingleData::from(s)),
            ..Default::default()
        }
    }
}

impl From<HashMap<String, String>> for Data {
    fn from(hm: HashMap<String, String>) -> Self {
        Data {
            value: DataType::AssocArray(AssocData::from(hm)),
            ..Default::default()
        }
    }
}

impl From<String> for DataType {
    fn from(s: String) -> Self {
        DataType::Single(SingleData::from(s))
    }
}

impl From<Vec<String>> for DataType {
    fn from(vals: Vec<String>) -> Self {
        DataType::Array(ArrayData::from(vals))
    }
}

impl From<&Vec<String>> for DataType {
    fn from(vals: &Vec<String>) -> Self {
        DataType::Array(ArrayData::from(vals.clone()))
    }
}

impl From<HashMap<String, String>> for DataType {
    fn from(hm: HashMap<String, String>) -> Self {
        DataType::AssocArray(AssocData::from(hm))
    }
}


impl From<Vec<String>> for Data {
    fn from(vals: Vec<String>) -> Self {
        Data {
            value: DataType::Array(ArrayData::from(vals)),
            ..Default::default()
        }
    }
}

impl Data {
    pub fn get_value(&mut self) -> DataType {
        match &mut self.value {
            DataType::Special(d) => d.update(),
            _ => self.value.clone(),
        }
    }

    pub fn set_assoc_elem(&mut self, key: &String, val: &String) -> bool {
        match &mut self.value {
            DataType::AssocArray(a) => a.set(key.to_string(), val.to_string()),
            _ => return false,
        }
    }

    pub fn set_array_elem(&mut self, pos: usize, val: &String) -> bool {
        match &mut self.value {
            DataType::Array(a) => a.set(pos, val), 
            _ => return false,
        }
    }
}
