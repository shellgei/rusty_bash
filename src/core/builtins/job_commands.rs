//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod bg;
pub mod disown;
pub mod fg; 
pub mod jobs; 
pub mod kill; 
pub mod wait; 

use libc;
use crate::error::exec::ExecError;
use crate::core::JobEntry;
use crate::utils::arg;
use crate::ShellCore;
use std::{thread, time};
use std::sync::atomic::Ordering::Relaxed;

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

fn print(core: &mut ShellCore, args: &[String]) {
    let l_opt = arg::has_option("-l", args);
    let r_opt = arg::has_option("-r", args);
    let s_opt = arg::has_option("-s", args);

    let mut rem = vec![];
    for id in 0..core.job_table.len() {
        if core.job_table[id].print(&core.job_table_priority, l_opt, r_opt, s_opt, true) {
            rem.push(id);
        }
    }

    for pos in rem.into_iter().rev() {
        remove(core, pos);
    }
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

fn wait_next(
    core: &mut ShellCore,
    ids: &[usize],
    var_name: &Option<String>,
    f_opt: bool,
) -> Result<(i32, bool), ExecError> {
    if core.job_table_priority.is_empty() {
        return Ok((127, false));
    }

    let mut exit_status = 0;
    let mut drop = 0;
    let mut end = false;
    let mut pid = String::new();
    let mut remove_job = false;

    loop {
        if core.sigint.load(Relaxed) {
            return Ok((130, false));
        }

        thread::sleep(time::Duration::from_millis(10)); //0.01秒周期に変更
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if !ids.contains(&i) && !ids.is_empty() {
                continue;
            }

            let es = job.update_status(false, true)?;
            //if let Ok(es) = job.update_status(false, true) {
                if job.display_status == "Done"
                    || job.display_status == "Killed"
                    || (job.display_status == "Stopped" && !f_opt)
                {
                    exit_status = es;
                    drop = i;
                    end = true;
                    remove_job = job.display_status == "Done" || job.display_status == "Killed";
                    pid = job.pids[0].to_string();
                    break;
                }
            //}
        }

        if end {
            break;
        }
    }

    if let Some(var) = var_name {
        let _ = core.db.unset(var, None, false);
        if let Err(e) = core.db.set_param(var, &pid, None) {
            e.print(core);
        }
    }

    if remove_job {
        remove(core, drop);
    }
    Ok((exit_status, true))
}

fn wait_pid(core: &mut ShellCore, pid: i32, var_name: &Option<String>, f_opt: bool) -> Result<(i32, bool), ExecError> {
    match pid_to_array_pos(pid, &core.job_table) {
        Some(i) => wait_a_job(core, i, var_name, f_opt),
        None => Ok((127, false)),
    }
}

fn wait_a_job(
    core: &mut ShellCore,
    pos: usize,
    var_name: &Option<String>,
    f_opt: bool,
) -> Result<(i32, bool), ExecError> {
    if core.job_table.len() < pos {
        return Ok((
            super::error_(127, "wait", "invalpos jobpos", core),
            false,
        ));
    }


    let ans = core.job_table[pos].nonblock_wait(&mut core.sigint)?;
    if let Some(var) = var_name {
          let _ = core.db.unset(var, None, false);
          let pid = core.job_table[pos].pids[0].to_string();
           core.db.set_param(var, &pid, None)?;
    }

    if f_opt && core.job_table[pos].display_status == "Stopped" {
        wait_a_job(core, pos, var_name, f_opt)
    } else {
        remove(core, pos);
        Ok(ans)
    }
}

