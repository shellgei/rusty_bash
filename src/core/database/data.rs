//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod single;

use crate::error::exec::ExecError;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Data {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_print_string()).finish()
    }
}

pub trait Data {
    fn boxed_clone(&self) -> Box<dyn Data>;
    fn get_print_string(&self) -> String;

    fn set_as_single(&mut self, _: &str, _: &str) -> Result<(), ExecError> {
        Err(ExecError::Other("Undefined call set_as_single".to_string()))
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Err(ExecError::Other("not a single variable".to_string()))
    }

    fn set_flag(&mut self, _: char) {}
}
