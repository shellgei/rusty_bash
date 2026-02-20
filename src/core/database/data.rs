//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

pub mod epochrealtime;
pub mod epochseconds;
pub mod random;
pub mod srandom;
pub mod seconds;
pub mod single;

use crate::error::exec::ExecError;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Data {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self._get_fmt_string()).finish()
    }
}

pub trait Data {
    fn boxed_clone(&self) -> Box<dyn Data>;

    fn _get_fmt_string(&self) -> String {
        "*****".to_string()
    }

    fn get_fmt_string(&mut self) -> String {
        self._get_fmt_string()
    }

    fn set_as_single(&mut self, name: &str, _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Err(ExecError::Other("not a single variable".to_string()))
    }

    fn readonly_check(&mut self, name: &str) -> Result<(), ExecError> {
        if self.has_flag('r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }
        Ok(())
    }

    fn set_flag(&mut self, _: char) {}

    fn get_flags(&self) -> &str;

    fn has_flag(&mut self, flag: char) -> bool {
        self.get_flags().contains(flag)
    }
}
