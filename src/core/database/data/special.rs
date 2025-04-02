//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::{DataBase, HashMap};
use crate::error::exec::ExecError;
use crate::utils::random::get_random;
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

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok( (self.function)(&mut self.internal_data) )
    }

    fn len(&mut self) -> usize {
        let v = (self.function)(&mut self.internal_data);
        v.chars().count()
    }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        if self.function == get_random {
            self.internal_data.clear();
            self.internal_data.push(value.to_string());
        }
        Ok(())
    }

    fn is_special(&self) -> bool {true}
}

impl SpecialData {
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str,
                                f: fn(&mut Vec<String>)-> String)-> Result<(), String> {
        db_layer.insert( name.to_string(), Box::new(SpecialData::from(f)) );
        Ok(())
    }

    pub fn get(db: &mut DataBase, name: &str) -> Option<String> {
        let num = db.params.len();
        for layer in (0..num).rev()  {
            if let Some(v) = db.params[layer].get_mut(name) {
                if v.is_special() {
                    if let Ok(s) = v.get_as_single() {
                        return Some(s);
                    }
                }
            }
        }
        None
    }
}
