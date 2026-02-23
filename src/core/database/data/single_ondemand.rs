//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{Data, ExecError};

#[derive(Debug, Clone)]
pub struct OnDemandSingle {
    generator: fn() -> String,
    flags: String,
}

impl Data for OnDemandSingle {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    
    fn get_fmt_string(&mut self) -> String {
        self.get_as_single().unwrap()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok((self.generator)())
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

impl OnDemandSingle {
    pub fn new(func: fn() -> String) -> Self {
        Self {
            generator: func,
            flags: "".to_string(),
        }
    }
}
