//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;

#[derive(Debug, Clone, Default)]
pub struct UninitArray {
    pub flags: String,
}

impl Data for UninitArray {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    fn get_print_string(&self) -> String {
        "".to_string()
    }

    fn clear(&mut self) {}
    fn is_initialized(&self) -> bool {
        false
    }
    fn get_as_array(&mut self, _: &str, _: &str) -> Result<String, ExecError> {
        Ok("".to_string())
    }
    fn get_all_as_array(&mut self, _: bool) -> Result<Vec<String>, ExecError> {
        Ok(vec![])
    }
    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(vec![])
    }
    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok("".to_string())
    }
    fn is_array(&self) -> bool {
        true
    }
    fn len(&mut self) -> usize {
        0
    }
    fn elem_len(&mut self, _: &str) -> Result<usize, ExecError> {
        Ok(0)
    }
    fn remove_elem(&mut self, _: &str) -> Result<(), ExecError> {
        Ok(())
    }

    fn set_flag(&mut self, flag: char) -> Result<(), ExecError> {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
        Ok(())
    }

    fn unset_flag(&mut self, flag: char) -> Result<(), ExecError> {
        self.flags.retain(|e| e != flag);
        Ok(())
    }

    fn has_flag(&mut self, flag: char) -> Result<bool, ExecError> {
        Ok(self.flags.contains(flag))
    }
}
