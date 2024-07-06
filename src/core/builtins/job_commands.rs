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

fn arg_to_id(s: &str, priority: &Vec<usize>) -> usize {
    if s == "%+" {
        return match priority.len() {
            0 => 0, 
            _ => priority[0],
        };
    }

    if s == "%-" {
        return match priority.len() {
            0 => 0, 
            1 => 0, 
            _ => priority[1],
        };
    }

    0
}

pub fn bg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let id = if args.len() == 1 {
        if core.job_table_priority.len() == 0 {
            return 1;
        }
        core.job_table_priority[0]
    }else if args.len() == 2 {
        arg_to_id(&args[1], &core.job_table_priority)
    }else{
        return 1;
    };

    match id_to_job(id, &mut core.job_table) {
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
