//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data2;

#[derive(Debug, Clone)]
pub struct SpecialData2 {
    pub internal_data: Vec<String>,
    pub function: fn(&mut Vec<String>) -> String,
}

/*
impl From<fn(&mut Vec<String>)-> String> for SpecialData {
    fn from(f: fn(&mut Vec<String>)-> String) -> SpecialData {
        SpecialData {
            internal_data: vec![],
            function: f,
        }
    }
}

impl SpecialData {
    pub fn update(&mut self) -> DataType {
        let ans = (self.function)(&mut self.internal_data);
        DataType::from(ans)
    }
}

impl From<Vec<String>> for ArrayData2 {
    fn from(v: Vec<String>) -> Self {
        Self { body: v }
    }
}
*/

impl Data2 for SpecialData2 {
    fn boxed_clone(&self) -> Box<dyn Data2> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        self.internal_data.join(" ")
    }

    fn set_as_array(&mut self, key: &str, value: &str) -> bool { false }
    fn get_as_array(&mut self, key: &str) -> Option<String> { None }
    fn get_all_as_array(&mut self) -> Option<Vec<String>> { None }

    fn set_as_single(&mut self, value: &str) -> bool {false}
    fn get_as_single(&mut self) -> Option<String> {
        Some( (self.function)(&mut self.internal_data) )
    }

    fn len(&mut self) -> usize {
        let v = (self.function)(&mut self.internal_data);
        v.chars().count()
    }
}
/*

impl ArrayData2 {
    /*
    pub fn get(&self, key: usize) -> Option<String> {
        match key < self.body.len() {
            true  => Some(self.body[key].clone()),
            false => None,
        }
    }

    pub fn values(&self) -> Vec<String> {
        self.body.clone()
    }*/
}
*/
