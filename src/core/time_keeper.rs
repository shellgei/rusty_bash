//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

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
