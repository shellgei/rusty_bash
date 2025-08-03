//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use crate::utils;

#[derive(Debug, Clone)]
pub struct IntData {
    pub body: isize,
}

impl Data for IntData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    fn print_body(&self) -> String {
        utils::to_ansi_c(&self.body.to_string())
    }

    fn clear(&mut self) {}

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        match value.parse::<isize>() {
            Ok(n) => self.body = n,
            Err(e) => {
                return Err(ExecError::Other(e.to_string()));
            }
        }
        Ok(())
    }

    fn append_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        match value.parse::<isize>() {
            Ok(n) => self.body += n,
            Err(e) => {
                return Err(ExecError::Other(e.to_string()));
            }
        }
        Ok(())
    }

    fn init_as_num(&mut self) -> Result<(), ExecError> {
        self.body = 0;
        Ok(())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok(self.body.to_string())
    }
    fn get_as_single_num(&mut self) -> Result<isize, ExecError> {
        Ok(self.body)
    }

    fn len(&mut self) -> usize {
        self.body.to_string().len()
    }
    fn is_single(&self) -> bool {
        true
    }
    fn is_single_num(&self) -> bool {
        true
    }
}
