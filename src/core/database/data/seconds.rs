//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use nix::time;
use nix::time::ClockId;
use std::time::Duration;
use super::Data;

fn monotonic_time() -> Duration {
    let now = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    Duration::new(
        now.tv_sec().try_into().unwrap(),
        now.tv_nsec().try_into().unwrap(),
    )
}

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

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let diff = monotonic_time() - self.origin;
        let ans = format!("{}", diff.as_secs() as isize + self.shift);
        Ok(ans)
    }

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        self.shift = value.parse::<isize>().unwrap_or(0);
        self.origin = monotonic_time();
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
            origin: monotonic_time(),
            shift: 0,
            flags: "i".to_string(),
        }
    }
}
