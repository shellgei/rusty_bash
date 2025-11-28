//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use crate::utils::clock;

#[derive(Debug, Clone)]
pub struct EpochRealTime {
    flags: String,
}

impl Data for EpochRealTime {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    fn get_print_string(&self) -> String {
        "".to_string()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok(clock::get_epochrealtime())
    }

    fn len(&mut self) -> usize {
        self.get_as_single().unwrap_or_default().len()
    }
    fn set_as_single(&mut self, name: &str, _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)
    }

    fn is_special(&self) -> bool {
        true
    }
    fn is_single_num(&self) -> bool {
        true
    }

    fn set_flag(&mut self, flag: char) {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
    }

    fn unset_flag(&mut self, flag: char) {
        self.flags.retain(|e| e != flag);
    }

    fn has_flag(&mut self, flag: char) -> bool {
        self.flags.contains(flag)
    }
}

impl EpochRealTime {
    pub fn new() -> Self {
        Self { flags: "i".to_string() }
    }
}
