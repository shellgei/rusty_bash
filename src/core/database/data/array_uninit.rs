//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use super::Data;

#[derive(Debug, Clone, Default)]
pub struct UninitArray { }

/*
impl From<Option<Vec<String>>> for UninitArray {
    fn from(v: Option<Vec<String>>) -> Self {
        return Self { }
    }
}*/

impl Data for UninitArray {
    fn boxed_clone(&self) -> Box<dyn Data> { Box::new(self.clone()) }
    fn print_body(&self) -> String {"".to_string()}

    fn clear(&mut self) { }
    fn is_initialized(&self) -> bool { false }
    fn get_as_array(&mut self, _: &str) -> Result<String, ExecError> { Ok( "".to_string() ) }
    fn get_all_as_array(&mut self) -> Result<Vec<String>, ExecError> { Ok(vec![]) }
    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> { Ok(vec![]) }
    fn get_as_single(&mut self) -> Result<String, ExecError> { Ok("".to_string()) }
    fn is_array(&self) -> bool {true}
    fn len(&mut self) -> usize { 0 }
    fn elem_len(&mut self, _: &str) -> Result<usize, ExecError> { Ok(0) }
    fn remove_elem(&mut self, _: &str) -> Result<(), ExecError> { Ok(()) }
}
