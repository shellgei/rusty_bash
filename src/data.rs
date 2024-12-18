//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
//pub mod assoc;
pub mod single;
pub mod special;

use self::array::ArrayData;
use self::single::SingleData;
use self::special::SpecialData;

#[derive(Debug, Clone, Default)]
pub enum DataType {
    #[default]
    None,
    Special(SpecialData),
    Single(SingleData),
    //AssocArray(AssocData),
    Array(ArrayData),
}

#[derive(Debug, Clone, Default)]
pub struct Data {
    pub body: DataType,
    pub attributes: String,
}

impl From<DataType> for Data {
    fn from(v: DataType) -> Self {
        Data {
            body: v,
            ..Default::default()
        }
    }
}

impl From<&str> for Data {
    fn from(s: &str) -> Self {
        Data {
            body: DataType::Single(SingleData::from(s)),
            ..Default::default()
        }
    }
}

/*
impl From<HashMap<String, String>> for Data {
    fn from(hm: HashMap<String, String>) -> Self {
        Data {
            body: DataType::AssocArray(AssocData::from(hm)),
            ..Default::default()
        }
    }
}*/

impl From<fn(&mut Vec<String>)-> String> for Data {
    fn from(f: fn(&mut Vec<String>)-> String) -> Data {
        Data {
            body: DataType::Special(SpecialData::from(f)),
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

/*
impl From<HashMap<String, String>> for DataType {
    fn from(hm: HashMap<String, String>) -> Self {
        DataType::AssocArray(AssocData::from(hm))
    }
}*/


impl From<Vec<String>> for Data {
    fn from(vals: Vec<String>) -> Self {
        Data {
            body: DataType::Array(ArrayData::from(vals)),
            ..Default::default()
        }
    }
}

impl Data {
    /*
    pub fn get_body(&mut self) -> DataType {
        match &mut self.body {
            DataType::Special(d) => d.update(),
            _ => self.body.clone(),
        }
    }*/

    pub fn set_array_elem(&mut self, pos: usize, val: &String) -> bool {
        match &mut self.body {
            DataType::Array(a) => a.set(pos, val), 
            _ => return false,
        }
    }
}
