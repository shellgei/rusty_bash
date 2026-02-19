//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{Data, ExecError};
use crate::utils::clock;

#[derive(Debug, Clone, Default)]
pub struct EpochRealtime {
    flags: String,
}

impl Data for EpochRealtime {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    
    fn get_fmt_string(&mut self) -> String {
        self.get_as_single().unwrap()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok(clock::get_epochrealtime())
    }

    fn set_as_single(&mut self, name: &str, _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)
    }

    fn set_flag(&mut self, flag: char) {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
    }

    fn get_flags(&self) -> &str {
        &self.flags
    }
}
