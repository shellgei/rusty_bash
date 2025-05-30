//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod array_int;
pub mod array_uninit;
pub mod assoc;
pub mod epochseconds;
pub mod epochrealtime;
pub mod random;
pub mod seconds;
pub mod single;
pub mod single_int;
//pub mod special;
pub mod srandom;

use crate::error::arith::ArithError;
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

    fn print_with_name(&self, name: &str, declare_print: bool) {
        if self.is_special() {
            println!("{}", name);
            return;
        }

        let body = self.print_body();
        if ! self.is_initialized() {
            println!("{}", name);
        }else if declare_print && self.is_single() {
            println!("{}=\"{}\"", name, body);
        }else{
            println!("{}={}", name, body);
        }
    }

    fn is_initialized(&self) -> bool {true}

    //fn push_elems(&mut self, _: Vec<String>) -> Result<(), ExecError> { Err(ExecError::Other("Undefined call push_elems".to_string())) }

    fn clear(&mut self) {}
    fn set_as_single(&mut self, _: &str) -> Result<(), ExecError> {Err(ExecError::Other("Undefined call set_as_single".to_string()))}
    fn append_as_single(&mut self, _: &str) -> Result<(), ExecError> {Err(ExecError::Other("Undefined call set_as_single".to_string()))}
    fn get_as_single_num(&mut self) -> Result<isize, ExecError> {Err(ExecError::Other("not a single variable".to_string()))}

    fn set_as_array(&mut self, _: &str, _: &str) -> Result<(), ExecError> {
        Err(ExecError::Other("not an array".to_string()))
    }
    fn append_to_array_elem(&mut self, _: &str, _: &str) -> Result<(), ExecError> {
        Err(ExecError::Other("not an array".to_string()))
    }

    fn set_as_assoc(&mut self, _: &str, _: &str) -> Result<(), ExecError> {
        Err(ExecError::Other("not an associative table".to_string()))
    }
    fn append_to_assoc_elem(&mut self, _: &str, _: &str) -> Result<(), ExecError> {
        Err(ExecError::Other("not an associative table".to_string()))
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {Err(ExecError::Other("not a single variable".to_string()))}
    fn get_as_array(&mut self, key: &str) -> Result<String, ExecError> { //TODO: change to ArithError
        let err = ArithError::OperandExpected(key.to_string());
        Err(ExecError::ArithError(key.to_string(), err)) 
    }
    fn get_as_assoc(&mut self, key: &str) -> Result<String, ExecError> {
        let err = ArithError::OperandExpected(key.to_string());
        Err(ExecError::ArithError(key.to_string(), err)) 
    }

    fn get_as_array_or_assoc(&mut self, pos: &str) -> Result<String, ExecError> {
        if self.is_assoc() {
            return match self.get_as_assoc(pos) {
                Ok(d)  => Ok(d),
                Err(_) => Ok("".to_string()),
            }
        }
        if self.is_array() {
            return match self.get_as_array(pos) {
                Ok(d)  => Ok(d),
                Err(_) => Ok("".to_string()),
            }
        }

        if pos == "0" || pos == "*" || pos == "@" {
            return match self.get_as_single() {
                Ok(d)  => Ok(d),
                Err(_) => Ok("".to_string()),
            }
        }else{
            return Ok("".to_string());
        }

        //Err(ExecError::ArrayIndexInvalid(pos.to_string()))
    }

    fn get_all_as_array(&mut self, _: bool) -> Result<Vec<String>, ExecError> {
        Err(ExecError::Other("not an array".to_string()))
    }


    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {Err(ExecError::Other("not an array".to_string()))}

    fn is_special(&self) -> bool {false}
    fn is_single(&self) -> bool {false}
    fn is_single_num(&self) -> bool {false}
    fn is_assoc(&self) -> bool {false}
    fn is_array(&self) -> bool {false}
    fn len(&mut self) -> usize;

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        match key {
            "0" => Ok(self.len()),
            _   => Ok(0),
        }
    }

    fn init_as_num(&mut self) -> Result<(), ExecError> {Err(ExecError::Other("Undefined call init_as_num".to_string()))}

    fn remove_elem(&mut self, _: &str) -> Result<(), ExecError> {Err(ExecError::Other("Undefined call remove_elem".to_string()))}
}
