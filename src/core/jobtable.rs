//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::unistd::Pid;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<Pid>,
    statuses: Vec<WaitStatus>,
    text: String,
    change: bool,
}

fn wait_nonblock(pid: &Pid, status: &mut WaitStatus) {
    match waitpid(*pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(s) => *status = s,
        _  => panic!("SUSHI INTERNAL ERROR (wrong pid wait)"),
    }
}

fn still(status: &WaitStatus) -> bool {
    match &status {
        WaitStatus::StillAlive => true,
        _ => false,
    }
}

impl JobEntry {
    pub fn new(pids: Vec<Option<Pid>>, text: &str) -> JobEntry {
        let len = pids.len();
        JobEntry {
            pids: pids.into_iter().flatten().collect(),
            statuses: vec![ WaitStatus::StillAlive; len ],
            text: text.to_string(),
            change: false,
        }
    }

    pub fn update_status(&mut self) {
        let before = self.statuses[0];
        for (status, pid) in self.statuses.iter_mut().zip(&self.pids) {
            if still(status) {
                wait_nonblock(pid, status);
            }
        }
        self.change |= before != self.statuses[0];
    }

    pub fn print(&self) {
        eprintln!("{:?}     {}", &self.statuses[0], &self.text);
    }
}

impl ShellCore {
    pub fn jobtable_check_status(&mut self) {
        for e in self.job_table.iter_mut() {
            e.update_status();
        }
    }

    pub fn jobtable_print_status_change(&mut self) {
        for e in self.job_table.iter_mut().filter(|e| e.change) {
            e.print();
            e.change = false;
        }

        self.job_table.retain(|e| still(&e.statuses[0]));
    }
}
