//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use nix::sys::wait::WaitStatus;

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<Pid>,
    statuses: Vec<WaitStatus>,
    text: String,
}

impl JobEntry {
    pub fn new(pids: Vec<Option<Pid>>, text: &str) -> JobEntry {
        let len = pids.len();
        JobEntry {
            pids: pids.into_iter().flatten().collect(),
            statuses: vec![ WaitStatus::StillAlive; len ],
            text: text.to_string(),
        }
    }
}
