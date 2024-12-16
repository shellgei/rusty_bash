//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

//use crate::data::{Data, DataType};

#[derive(Debug, Clone)]
pub struct SpecialData {
    pub internal_data: Vec<String>,
    pub dynamic_get: fn(&mut Vec<String>) -> String,
}

/*
impl SpecialData {
    pub fn update(&mut self) -> DataType {
        let ans = (self.dynamic_get)(self);
        DataType::from(ans)
    }
}*/
