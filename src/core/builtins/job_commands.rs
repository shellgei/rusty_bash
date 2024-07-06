//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::JobEntry;

fn id_to_job(id: usize, jobs: &mut Vec<JobEntry>) -> Option<&mut JobEntry> {
    for job in jobs.iter_mut() {
        if job.id == id {
            return Some(job);
        }
    }

    None
}

pub fn bg(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    if core.job_table_priority.len() == 0 {
        return 1;
    }

    match id_to_job(core.job_table_priority[0], &mut core.job_table) {
        Some(job) => job.send_cont(),
        _ => return 1, 
    }
    0
}

pub fn jobs(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    for job in core.job_table.iter() {
        job.print(&core.job_table_priority);
    }
    0
}
