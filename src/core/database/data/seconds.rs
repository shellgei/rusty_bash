//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils::clock;
use std::time::Duration;
use super::{Data, ExecError};

#[derive(Debug, Clone)]
pub struct Seconds {
    origin: Duration,
    shift: isize,
    flags: String,
}

impl Data for Seconds {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn get_fmt_string(&mut self) -> String {
        self.get_as_single().unwrap()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let diff = clock::monotonic_time() - self.origin;
        let ans = format!("{}", diff.as_secs() as isize + self.shift);
        Ok(ans)
    }

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        self.shift = value.parse::<isize>().unwrap_or(0);
        self.origin = clock::monotonic_time();
        Ok(())
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

impl Seconds {
    pub fn new() -> Self {
        Self {
            origin: clock::monotonic_time(),
            shift: 0,
            flags: "i".to_string(),
        }
    }
}
