//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
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

impl ShellCore {
    pub fn jobtable_entry(&mut self, pids :&Vec<Option<Pid>>) {
        let ps = pids.iter().map(|e| e.expect("")).collect();
        self.job_table.push(JobEntry::new(&ps));
    }

    pub fn jobtable_check(&mut self) {
        for e in self.job_table.iter_mut() {
            dbg!("{:?}", &e);
        }
    }
}
