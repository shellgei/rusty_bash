//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;

#[derive(Debug, Clone, PartialEq)]
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

fn process_still_alive(pid: &Pid) -> bool {
    match waitpid(*pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::StillAlive) => true,
        Ok(_)                      => false,
        _  => panic!("SUSHI INTERNAL ERROR (wrong pid wait)"),
    }
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

    pub fn check_status(&mut self) {
        for (status, pid) in self.pid_statuses.iter_mut().zip(&self.pids) {
            if status != &mut JobStatus::Finished && ! process_still_alive(pid) {
                *status = JobStatus::Finished;
            }
        }
    }
}

impl ShellCore {
    pub fn jobtable_check_status(&mut self) {
        for e in self.job_table.iter_mut() {
            e.check_status();
        }
    }

    pub fn jobtable_print_finish(&mut self) {
        for e in self.job_table.iter_mut() {
            if e.pid_statuses.iter().all(|s| s == &mut JobStatus::Finished) {
                e.status = JobStatus::Finished;
                eprintln!("Done {}", e.text);
            }
        }

        self.job_table.retain(|e| e.status != JobStatus::Finished);
    }
}

