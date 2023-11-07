//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;

#[derive(Debug)]
enum JobStatus {
    Running,
    Finished,
}

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<Pid>,
    status: JobStatus,
}

fn process_still_alive(pid: &Pid) -> bool {
    match waitpid(*pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::StillAlive) => true,
        Ok(_)                      => false,
        _  => panic!("SUSHI INTERNAL ERROR (wrong pid wait)"),
    }
}

impl JobEntry {
    pub fn new(pids: Vec<Option<Pid>>) -> JobEntry {
        JobEntry {
            pids: pids.into_iter().flatten().collect(),
            status: JobStatus::Running,
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
                eprintln!("");
            }
        }
    }
}

