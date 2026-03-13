//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use nix::sys::resource;
use nix::sys::resource::UsageWho;
use nix::sys::time::{TimeSpec, TimeVal};
use nix::time;
use nix::time::ClockId;

pub struct TimeKeeper {
    pub real: Option<TimeSpec>,
    pub user: TimeVal,
    pub sys: TimeVal,
}

impl Default for TimeKeeper {
    fn default() -> Self {
        Self {
            real: None,
            user: TimeVal::new(0, 0),
            sys: TimeVal::new(0, 0),
        }
    }
}

impl TimeKeeper {
    pub fn set(&mut self, on: bool) {
        if ! on {
            self.real = None;
            return;
        }

        let sush_usage = resource::getrusage(UsageWho::RUSAGE_SELF).unwrap();
        let children_usage = resource::getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

        self.user = sush_usage.user_time() + children_usage.user_time();
        self.sys = sush_usage.system_time() + children_usage.system_time();
        self.real = Some(time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap());
    }

    pub fn print_diff(&self) {
        if self.real.is_none() {
            return;
        }

        let print_time_spec = |item, t: TimeSpec|
            eprintln!("\n{}\t{}m{}.{:09}s", item, t.tv_sec()/60, t.tv_sec()%60, t.tv_nsec());
        let print_time_val = |item, t: TimeVal|
            eprintln!("{}\t{}m{}.{:06}s", item, t.tv_sec()/60, t.tv_sec()%60, t.tv_usec());

        let real_end_time = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
        let sush_usage = resource::getrusage(UsageWho::RUSAGE_SELF).unwrap();
        let children_usage = resource::getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

        print_time_spec("real", real_end_time - self.real.unwrap());
        print_time_val("usr", sush_usage.user_time() + children_usage.user_time() - self.user);
        print_time_val("sys", sush_usage.system_time() + children_usage.system_time() - self.sys);
    }
}
