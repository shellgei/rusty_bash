//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;

use crate::error::exec::ExecError;

#[derive(Debug, Clone)]
pub struct SingleData {
    body: String,
    flags: String,
}

impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self {
            body: s.to_string(),
            flags: String::new(),
        }
    }
}

impl Data for SingleData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn _get_fmt_string(&self) -> String {
        self.body.clone()
    }

    fn set_as_single(&mut self, name: &str,
                     value: &str) -> Result<(), ExecError> {
        if self.flags.contains('r') {
            return Err(ExecError::VariableReadOnly(name.to_string()));
        }

        self.body = value.to_string();
        Ok(())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok(self.body.clone())
    }

    fn set_flag(&mut self, flag: char) {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
    }
}
