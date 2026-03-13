//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils::clock;
use nix::sys::time::TimeVal;
use std::time::Duration;

pub struct TimeKeeper {
    pub real: Option<Duration>,
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

        (self.user, self.sys) = clock::get_user_and_sys();
        self.real = Some(clock::monotonic_time());
    } 

    pub fn print_diff(&self) {
        if self.real.is_none() {
            return;
        }
    
        let print_duration = |item, t: Duration| 
            eprintln!("\n{}\t{}m{}.{:09}s", item, t.as_secs()/60, t.as_secs()%60, t.subsec_nanos());
        let print_time_val = |item, t: TimeVal|
            eprintln!("{}\t{}m{}.{:06}s", item, t.tv_sec()/60, t.tv_sec()%60, t.tv_usec());
    
        print_duration("real", clock::monotonic_time() - self.real.unwrap());

        let (user, sys) = clock::get_user_and_sys();
        print_time_val("usr", user - self.user);
        print_time_val("sys", sys - self.sys);
    }
}
