//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use crate::utils::clock;
use std::time::Duration;

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

    fn _get_fmt_string(&self) -> String {
        "".to_string() //TODO
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let elapsed = clock::monotonic_time() - self.origin;
        let ans = format!("{}", elapsed.as_secs() as isize + self.shift);

        Ok(ans)
    }

    fn len(&mut self) -> usize {
        0
    }

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        self.shift = value.parse::<isize>().unwrap_or(0);
        self.origin = clock::monotonic_time();
        Ok(())
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

    fn get_flags(&mut self) -> String {
        self.flags.clone()
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
