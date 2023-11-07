//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;

#[derive(Debug, Clone)]
enum JobStatus {
    Running,
    Finished,
}

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<Pid>,
    pid_statuses: Vec<JobStatus>,
    status: JobStatus,
    text: String,
}

impl JobEntry {
    pub fn new(pids: Vec<Option<Pid>>, text: &str) -> JobEntry {
        let len = pids.len();
        JobEntry {
            pids: pids.into_iter().flatten().collect(),
            pid_statuses: vec![ JobStatus::Running; len ],
            status: JobStatus::Running,
            text: text.to_string(),
        }
    }
}
