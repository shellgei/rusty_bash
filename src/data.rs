//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod single;

use self::array::ArrayData;
use self::single::SingleData;

#[derive(Debug, Clone, Default)]
pub enum DataType {
    #[default]
    None,
    Single(SingleData),
    Array(ArrayData),
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
