//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod array;
pub mod array_int;
pub mod assoc;
pub mod assoc_int;
pub mod epochrealtime;
pub mod epochseconds;
pub mod random;
pub mod seconds;
pub mod single;
pub mod single_int;
pub mod srandom;
pub mod uninit;

use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use std::fmt;
use std::fmt::Debug;

fn to_int(s: &str) -> Result<isize, ExecError> {
    match s.parse::<isize>() {
        Ok(n) => Ok(n),
        Err(e) => Err(ArithError::OperandExpected(e.to_string()).into()),
    }
}

fn case_change(flags: &str, text: &mut String) {
    if flags.contains('l') {
        *text = text.to_lowercase();
    }else if flags.contains('u') {
        *text = text.to_uppercase();
    }
}

impl Debug for dyn Data {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self._get_fmt_string()).finish()
    }
}

impl Clone for Box<dyn Data> {
    fn clone(&self) -> Box<dyn Data> {
        self.boxed_clone()
    }
}

pub trait Data {
    fn boxed_clone(&self) -> Box<dyn Data>;
    fn _get_fmt_string(&self) -> String;

    fn get_fmt_string(&mut self) -> String {
        self._get_fmt_string()
    }

    fn get_str_type(&self) -> Box<dyn Data> {
        self.boxed_clone()
    }

    fn is_initialized(&self) -> bool {
        true
    }

    fn initialize(&mut self) -> Option<Box<dyn Data>> {
        None
    }

    fn has_key(&mut self, _: &str) -> Result<bool, ExecError> {
        Ok(false)
    }

    fn clear(&mut self) {}

    fn set_as_single(&mut self, name: &str,  _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)
    }
    fn append_as_single(&mut self, name: &str, _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        Err(ExecError::Other("Undefined call set_as_single".to_string()))
    }
    fn get_as_single_num(&mut self) -> Result<isize, ExecError> {
        Err(ExecError::Other("not a single variable".to_string()))
    }

    fn set_as_array(&mut self, name: &str, _: &str, _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        Err(ExecError::Other("not an array".to_string()))
    }
    fn append_to_array_elem(&mut self, name: &str, _: &str,
                            _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        Err(ExecError::Other("not an array".to_string()))
    }

    fn set_as_assoc(&mut self, name: &str, _: &str, _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        Err(ExecError::Other("not an associative table".to_string()))
    }
    fn append_to_assoc_elem(&mut self, name: &str, _: &str,
                            _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        Err(ExecError::Other("not an associative table".to_string()))
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Err(ExecError::Other("not a single variable".to_string()))
    }
    fn get_as_array(&mut self, key: &str, _: &str) -> Result<String, ExecError> {
        //TODO: change to ArithError
        let err = ArithError::OperandExpected(key.to_string());
        Err(ExecError::ArithError(key.to_string(), err))
    }
    fn get_as_assoc(&mut self, key: &str) -> Result<String, ExecError> {
        let err = ArithError::OperandExpected(key.to_string());
        Err(ExecError::ArithError(key.to_string(), err))
    }

    fn get_as_array_or_assoc(&mut self, pos: &str, ifs: &str) -> Result<String, ExecError> {
        if self.is_assoc() {
            return match self.get_as_assoc(pos) {
                Ok(d) => Ok(d),
                Err(_) => Ok("".to_string()),
            };
        }
        if self.is_array() {
            return match self.get_as_array(pos, ifs) {
                Ok(d) => Ok(d),
                Err(_) => Ok("".to_string()),
            };
        }

        if pos == "0" || pos == "*" || pos == "@" {
            match self.get_as_single() {
                Ok(d) => Ok(d),
                Err(_) => Ok("".to_string()),
            }
        } else {
            Ok("".to_string())
        }
    }

    fn get_all_as_array(&mut self, flatten: bool) -> Result<Vec<String>, ExecError> {
        self.get_vec_from(0, flatten)
    }

    fn get_vec_from(&mut self, _: usize, _: bool) -> Result<Vec<String>, ExecError> {
        Err(ExecError::Other("not an array".to_string()))
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Err(ExecError::Other("not an array".to_string()))
    }

    fn is_special(&self) -> bool {
        false
    }
    fn is_single(&self) -> bool {
        false
    }
    fn is_single_num(&self) -> bool {
        false
    }
    fn is_assoc(&self) -> bool {
        false
    }
    fn is_array(&self) -> bool {
        false
    }
    fn len(&mut self) -> usize;
    fn index_based_len(&mut self) -> usize {
        self.len()
    }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        match key {
            "0" => Ok(self.len()),
            _ => Ok(0),
        }
    }

    fn init_as_num(&mut self) -> Result<(), ExecError> {
        Err(ExecError::Other("Undefined call init_as_num".to_string()))
    }

    fn remove_elem(&mut self, _: &str) -> Result<(), ExecError> {
        Err(ExecError::Other("Undefined call remove_elem".to_string()))
    }

    fn readonly_check(&mut self, name: &str) -> Result<(), ExecError> {
        if self.has_flag('r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        Ok(())
    }

    fn set_flag(&mut self, _: char) {}

    fn unset_flag(&mut self, _: char) {}

    fn has_flag(&mut self, _: char) -> bool {
        false
    }

    fn get_flags(&mut self) -> String;
}
