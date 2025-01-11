//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::HashMap;
use super::Data;

#[derive(Debug, Clone)]
pub struct SpecialData {
    pub internal_data: Vec<String>,
    pub function: fn(&mut Vec<String>) -> String,
}

impl From<fn(&mut Vec<String>)-> String> for SpecialData {
    fn from(f: fn(&mut Vec<String>)-> String) -> Self {
        Self {
            internal_data: vec![],
            function: f,
        }
    }
}

impl Data for SpecialData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        self.internal_data.join(" ")
    }

    fn get_as_single(&mut self) -> Option<String> {
        Some( (self.function)(&mut self.internal_data) )
    }

    fn len(&mut self) -> usize {
        let v = (self.function)(&mut self.internal_data);
        v.chars().count()
    }

    fn is_special(&self) -> bool {true}
}

impl SpecialData {
    pub fn set(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str,
                      f: fn(&mut Vec<String>)-> String)-> Result<(), String> {
        db_layer.insert( name.to_string(), Box::new(SpecialData::from(f)) );
        Ok(())
    }
}
