//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<Pid>,
    pid_statuses: Vec<WaitStatus>,
    status: WaitStatus,
    text: String,
}

fn wait_nonblock(pid: &Pid, status: &mut WaitStatus) {
    match waitpid(*pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(s) => *status = s,
        _  => panic!("SUSHI INTERNAL ERROR (wrong pid wait)"),
    }
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

    pub fn update_status(&mut self) {
        for (status, pid) in self.pid_statuses.iter_mut().zip(&self.pids) {
            if status == &mut WaitStatus::StillAlive {
                wait_nonblock(pid, status);
            }
        }

        if self.pid_statuses.iter().all(|s| *s != WaitStatus::StillAlive) {
            self.status = self.pid_statuses[0];
        }
    }
}

impl ShellCore {
    pub fn jobtable_check_status(&mut self) {
        for e in self.job_table.iter_mut() {
            e.update_status();
        }
    }

    pub fn jobtable_print_finish(&mut self) {
        for e in self.job_table.iter() {
            if e.status != WaitStatus::StillAlive {
                eprintln!("Done {}", e.text);
            }
        }

        self.job_table.retain(|e| e.status == WaitStatus::StillAlive);
    }
}
