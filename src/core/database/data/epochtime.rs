//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{Data, ExecError};

#[derive(Debug, Clone)]
pub struct EpochTime {
    timefunc: fn() -> String,
    flags: String,
}

impl Data for EpochTime {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    
    fn get_fmt_string(&mut self) -> String {
        self.get_as_single().unwrap()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok((self.timefunc)())
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

impl EpochTime {
    pub fn new(timefn: fn() -> String) -> Self {
        Self {
            timefunc: timefn,
            flags: "".to_string(),
        }
    }
}
