//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::data::DataType;

#[derive(Debug, Clone)]
pub struct SpecialData {
    pub internal_data: Vec<String>,
    pub function: fn(&mut Vec<String>) -> String,
}

impl SpecialData {
    pub fn update(&mut self) -> DataType {
        let ans = (self.function)(&mut self.internal_data);
        DataType::from(ans)
    }
}
