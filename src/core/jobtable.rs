//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use nix::sys::wait::WaitStatus;

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<Pid>,
    pid_statuses: Vec<WaitStatus>,
    status: WaitStatus,
    text: String,
}

impl JobEntry {
    pub fn new(pids: Vec<Option<Pid>>, text: &str) -> JobEntry {
        let len = pids.len();
        JobEntry {
            pids: pids.into_iter().flatten().collect(),
            pid_statuses: vec![ WaitStatus::StillAlive; len ],
            status: WaitStatus::StillAlive,
            text: text.to_string(),
        }
    }
}
