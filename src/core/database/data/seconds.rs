//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use nix::time;
use nix::time::ClockId;
use std::time::Duration;
use super::Data;

#[derive(Debug, Clone)]
pub struct Seconds {
    origin: String,
    shift: isize,
}

fn monotonic_time() -> Duration {
    let now = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    Duration::new(now.tv_sec().try_into().unwrap(), now.tv_nsec().try_into().unwrap())
}


impl Data for Seconds {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String {
        "".to_string() //TODO
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let part: Vec<&str> = self.origin.split('.').collect();
        let sec = part[0].parse::<u64>().unwrap();
        let nano = part[1].parse::<u32>().unwrap();
        let offset = Duration::new(sec, nano);
        let elapsed = monotonic_time() - offset;

        let ans = format!("{}", elapsed.as_secs() as isize + self.shift);

        Ok(ans)
    }

    fn len(&mut self) -> usize {
        0
    }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.shift = value.parse::<isize>()?;
        let time = monotonic_time();
        self.origin = format!("{}.{}", time.as_secs(), time.subsec_nanos());
        Ok(()) // TODO
    }

    fn is_special(&self) -> bool {true}
    fn is_single_num(&self) -> bool { true }
}

impl Seconds {
    pub fn new() -> Self {
        let time = monotonic_time();
        Self {
            origin: format!("{}.{}", time.as_secs(), time.subsec_nanos()),
            shift: 0,
        }
    }
}
