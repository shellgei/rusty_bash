//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils;
use crate::error::exec::ExecError;
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
    fn print_body(&self) -> String { 
        utils::to_ansi_c(&self.body.to_string())
    }

    fn clear(&mut self) { self.body.clear(); }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body = value.to_string();
        Ok(())
    }

    fn append_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body += &value;
        Ok(())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> { Ok(self.body.to_string()) }
    fn len(&mut self) -> usize { self.body.chars().count() }
    fn is_single(&self) -> bool {true}

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }
        Ok(key == "0")
    }
}

    /*
impl SingleData {
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str, value: &str)-> Result<(), ExecError> {
        db_layer.insert( name.to_string(), Box::new(SingleData::from(value)) );
        Ok(())
    }
}
    */
