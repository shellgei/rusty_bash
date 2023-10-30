//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::unistd::Pid;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};

enum ProcessStatus {
    Running,
}

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

fn process_still_alive(pid: Pid) -> bool {
    match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::StillAlive) => false,
        Ok(_)                      => true,
        _  => panic!("sush(fatal): waitpid error"),
    }
}

impl ShellCore {
    pub fn jobtable_entry(&mut self, pids :&Vec<Option<Pid>>) {
        let ps = pids.iter().map(|e| e.expect("")).collect();
        self.job_table.push(JobEntry::new(&ps));
    }

    pub fn jobtable_check(&mut self) {
        for e in self.job_table.iter_mut() {
            /*
            if e.pids.iter().all(|pid| process_still_alive(*pid)) {
                eprintln!("stop!");
            }*/
        }
    }
}
