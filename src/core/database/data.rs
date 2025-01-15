//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod assoc;
pub mod single;
pub mod special;

use crate::error::exec::ExecError;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Data {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.print_body()).finish()
    }
}

impl Clone for Box::<dyn Data> {
    fn clone(&self) -> Box<dyn Data> {
        self.boxed_clone()
    }
}

pub trait Data {
    fn boxed_clone(&self) -> Box<dyn Data>;
    fn print_body(&self) -> String;

    fn print_with_name(&self, name: &str) {
        if self.is_special() {
            return;
        }
        println!("{}={}", name, self.print_body());
    }

    fn set_as_single(&mut self, _: &str) -> Result<(), ExecError> {Err(ExecError::Other("Undefined call".to_string()))}
    fn set_as_array(&mut self, _: &str, _: &str) -> Result<(), ExecError> {Err(ExecError::Other("not an array".to_string()))}
    fn set_as_assoc(&mut self, _: &str, _: &str) -> Result<(), ExecError> {Err(ExecError::Other("not an associative table".to_string()))}

    fn get_as_single(&mut self) -> Result<String, ExecError> {Err(ExecError::Other("not a single variable".to_string()))}
    fn get_as_array(&mut self, key: &str) -> Result<String, ExecError> {
        Err(ExecError::OperandExpected(key.to_string()))
    }
    fn get_as_assoc(&mut self, key: &str) -> Result<String, ExecError> {
        Err(ExecError::OperandExpected(key.to_string()))
    }

    fn get_as_array_or_assoc(&mut self, pos: &str) -> Result<String, ExecError> {
        if self.is_assoc() {
            return self.get_as_assoc(pos);
        }
        if self.is_array() {
            return self.get_as_array(pos);
        }
        Err(ExecError::ArrayIndexInvalid(pos.to_string()))
    }

    fn get_all_as_array(&mut self) -> Result<Vec<String>, ExecError> {Err(ExecError::Other("not an array".to_string()))}

    fn is_special(&self) -> bool {false}
    fn is_single(&self) -> bool {false}
    fn is_assoc(&self) -> bool {false}
    fn is_array(&self) -> bool {false}
    fn len(&mut self) -> usize;
}
