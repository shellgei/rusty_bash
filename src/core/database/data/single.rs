//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::HashMap;
use std::env;
use super::Data;

#[derive(Debug, Clone)]
pub struct SingleData {
    body: String,
}

impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self { body: s.to_string() }
    }
}

impl Data for SingleData {
    fn boxed_clone(&self) -> Box<dyn Data> { Box::new(self.clone()) }
    fn print_body(&self) -> String { self.body.clone() }

    fn set_as_single(&mut self, value: &str) -> Result<(), String> {
        self.body = value.to_string();
        Ok(())
    }

    fn get_as_single(&mut self) -> Result<String, String> { Ok(self.body.clone()) }
    fn len(&mut self) -> usize { self.body.chars().count() }
    fn is_single(&self) -> bool {true}
}

impl SingleData {
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str, value: &str)-> Result<(), String> {
        db_layer.insert( name.to_string(), Box::new(SingleData::from(value)) );
        Ok(())
    }

    pub fn set_value(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str, val: &str) -> Result<(), String> {
        if env::var(name).is_ok() {
            env::set_var(name, val);
        }
    
        if db_layer.get(name).is_none() {
            SingleData::set_new_entry(db_layer, name, "")?;
        }
    
        db_layer.get_mut(name).unwrap().set_as_single(val)
    }
}
