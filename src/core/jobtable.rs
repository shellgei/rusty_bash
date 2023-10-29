//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;

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

pub struct JobTable {
    pub jobs: Vec<JobEntry>,
}

impl JobTable {
    pub fn new() -> JobTable {
        JobTable {
            jobs: vec![],
        }
    }

    pub fn entry(&mut self, pids :&Vec<Option<Pid>>) {
        let ps = pids.iter().map(|e| e.expect("")).collect();
        self.jobs.push(JobEntry::new(&ps));
    }
}
