//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;

#[derive(Debug)]
pub struct JobEntry {
    pub pids: Vec<Pid>,
}

impl JobEntry {
    pub fn new(pids: &Vec<Pid>) -> JobEntry {
        JobEntry {
            pids: pids.clone(),
        }
    }
}
