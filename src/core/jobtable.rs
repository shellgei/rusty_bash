//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;

#[derive(Debug, PartialEq)]
enum JobStatus {
    Running,
    Finished,
}

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<(Pid, JobStatus)>,
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
        JobEntry {
            pids: pids.into_iter().flatten().map(|e| (e, JobStatus::Running)).collect(),
            status: JobStatus::Running,
            text: text.to_string(),
        }
    }

    pub fn check_status(&mut self) {
        if ! self.pids.iter().any(|p| process_still_alive(p)) {
            self.status = JobStatus::Finished;
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
            if e.status == JobStatus::Finished {
                eprintln!("Done {}", e.text);
            }
        }

        self.job_table.retain(|e| e.status != JobStatus::Finished);
    }
}

