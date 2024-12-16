//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod assoc;
pub mod single;
pub mod special;

use crate::core::HashMap;
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
pub struct Variable {
    pub value: DataType,
    pub attributes: String,
}

impl From<DataType> for Variable {
    fn from(v: DataType) -> Self {
        Variable {
            value: v,
            ..Default::default()
        }
    }
}

impl From<&str> for Variable {
    fn from(s: &str) -> Self {
        Variable {
            value: DataType::Single(SingleData::from(s)),
            ..Default::default()
        }
    }
}

impl From<HashMap<String, String>> for Variable {
    fn from(hm: HashMap<String, String>) -> Self {
        Variable {
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


impl From<Vec<String>> for Variable {
    fn from(vals: Vec<String>) -> Self {
        Variable {
            value: DataType::Array(ArrayData::from(vals)),
            ..Default::default()
        }
    }
}

impl Variable {
    pub fn set_data(&mut self, data: String) {
        match &mut self.value {
            DataType::Single(s) => s.data = data,
            DataType::Special(s) => s.data = data,
            _ => {},
        }
    }

    pub fn get_value(&mut self) -> DataType {
        match &self.value {
            DataType::Special(d) => {
                let ans = (d.dynamic_get)(self);
                DataType::from(ans)
            },
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

    /*
    fn print_data(&self, k: &str, core: &mut ShellCore) {
        match self.get_value(k) {
            Some(DataType::Single(s)) => {
                println!("{}={}", k.to_string(), s.data.to_string()); 
            },
            Some(DataType::Array(a)) => {
                let mut formatted = String::new();
                formatted += "(";
                for i in 0..a.len() {
                    let val = a.get(i).unwrap_or("".to_string());
                    formatted += &format!("[{}]=\"{}\" ", i, val).clone();
                };
                if formatted.ends_with(" ") {
                    formatted.pop();
                }
                formatted += ")";
                println!("{}={}", k.to_string(), formatted); 
            },
            Some(DataType::AssocArray(a)) => {
                let mut formatted = String::new();
                formatted += "(";
                for k in a.keys() {
                    let v = a.get(&k).unwrap_or("".to_string());
                    formatted += &format!("[{}]=\"{}\" ", k, v);
                }
                if formatted.ends_with(" ") {
                    formatted.pop();
                }
                formatted += ")";
                println!("{}={}", k.to_string(), formatted); 
            },
            _ => {},
        }
    }
    */
}
