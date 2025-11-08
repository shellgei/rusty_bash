//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use crate::utils::clock;

#[derive(Debug, Clone)]
pub struct EpochRealTime {}

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
    fn set_as_single(&mut self, _: &str) -> Result<(), ExecError> {
        Ok(())
    }

    fn is_special(&self) -> bool {
        true
    }
    fn is_single_num(&self) -> bool {
        true
    }
}
