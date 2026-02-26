//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod bg;
pub mod disown;
pub mod fg; 
pub mod jobs; 
pub mod kill; 
pub mod wait; 

use libc;
use crate::core::JobEntry;
use crate::ShellCore;

pub fn set(core: &mut ShellCore) {
    core.builtins.insert("jobs".to_string(), jobs::jobs);
    core.builtins.insert("kill".to_string(), kill::kill);
    core.builtins.insert("wait".to_string(), wait::wait);
    core.builtins.insert("disown".to_string(), disown::disown);
    core.builtins.insert("bg".to_string(), bg::bg);
    core.builtins.insert("fg".to_string(), fg::fg);
}

fn pid_to_array_pos(pid: i32, jobs: &[JobEntry]) -> Option<usize> {
    (0..jobs.len()).find(|&i| jobs[i].pids[0].as_raw() == pid)
}

fn jobid_to_pos(id: usize, jobs: &mut [JobEntry]) -> Option<usize> {
    for (i, job) in jobs.iter_mut().enumerate() {
        if job.id == id {
            return Some(i);
        }
    }
    None
}

fn jobspec_to_array_pos(core: &mut ShellCore, com: &str, jobspec: &str) -> Option<usize> {
    let poss = jobspec_to_array_poss(core, jobspec);
    if poss.is_empty() {
        let msg = format!("{}: no such job", &jobspec);
        super::error_(127, com, &msg, core);
        return None;
    } else if poss.len() > 1 {
        let msg = format!(
            "{}: ambiguous job spec",
            jobspec.strip_prefix('%').unwrap_or(jobspec)
        );
        super::error_(127, com, &msg, core);
        return None;
    }

    Some(poss[0])
}

fn jobspec_to_array_poss(core: &mut ShellCore, jobspec: &str) -> Vec<usize> {
    if jobspec.is_empty() {
        return (0..core.job_table.len()).collect();
    }

    if core.job_table.is_empty() {
        return vec![];
    }

    let s = &jobspec[1..];
    let mut ans = vec![];

    if let Ok(n) = s.parse::<usize>() {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if n == job.id {
                ans.push(i);
            }
        }
    } else if s == "%" || s == "+" {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if job.id == core.job_table_priority[0] {
                ans.push(i);
            }
        }
    } else if s == "-" {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if core.job_table_priority.len() < 2 {
                if job.id == core.job_table_priority[0] {
                    ans.push(i);
                }
            } else if job.id == core.job_table_priority[1] {
                ans.push(i);
            }
        }
    } else if let Some(stripped) = s.strip_prefix('?') {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if job.text.contains(stripped) {
                ans.push(i);
            }
        }
    } else {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if job.text.starts_with(s) {
                ans.push(i);
            }
        }
    }

    ans
}

fn remove_coproc(core: &mut ShellCore, pos: usize) {
    if let Some(name) = &core.job_table[pos].coproc_name {
        let _ = core.db.unset(&name, None, false);
        let _ = core.db.unset(&(name.to_owned() + "_PID"), None, false);

        if let Ok(fd0) = core.db.get_elem(&name, "0") {
            if let Ok(n) = fd0.parse::<i32>() {
                let _ = unsafe{libc::close(n)};
            }
        }
        if let Ok(fd1) = core.db.get_elem(&name, "1") {
            if let Ok(n) = fd1.parse::<i32>() {
                let _ = unsafe{libc::close(n)};
            }
        }

        let _ = core.db.unset(&(name), None, false);
    }
}

fn remove(core: &mut ShellCore, pos: usize) {
    let job_id = core.job_table[pos].id;
    remove_coproc(core, pos);
    core.job_table.remove(pos);
    core.job_table_priority.retain(|id| *id != job_id);
}

