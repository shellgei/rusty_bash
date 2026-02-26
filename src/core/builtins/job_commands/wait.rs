//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::builtins;
use crate::error::exec::ExecError;
use crate::utils::arg;
use std::{thread, time};
use std::sync::atomic::Ordering::Relaxed;

pub fn wait(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    if core.is_subshell {
        builtins::error_(127, &args[0], "called from subshell", core);
    }

    if args.len() <= 1 {
        match wait_all(core) {
            Ok(n) => return n,
            Err(e) => {
                e.print(core);
                return 1;
            },
        }
    }

    let mut args = arg::dissolve_options(&args);
    let var_name = arg::consume_with_next_arg("-p", &mut args);
    let f_opt = arg::consume_arg("-f", &mut args);

    if args.len() > 1 && args[1] == "-n" {
        match wait_n(core, &mut args, &var_name, f_opt) {
            Ok(n) => return n,
            Err(e) => {
                e.print(core);
                return 1;
            },
        }
    }

    if args.len() > 1 {
        match wait_arg_job(core, &args[0], &args[1], &var_name, f_opt) {
            Ok(n) => return n.0,
            Err(e) => {
                e.print(core);
                return 1;
            },
        }
    }
    1
}

fn wait_n(
    core: &mut ShellCore,
    args: &mut Vec<String>,
    var_name: &Option<String>,
    f_opt: bool,
) -> Result<i32, ExecError> {
    let mut jobs = arg::consume_with_subsequents("-n", args);
    jobs.remove(0);
    if jobs.is_empty() {
        return Ok(wait_next(core, &[], var_name, f_opt)?.0);
    }

    let mut ids = vec![];
    for j in &jobs {
        if j.starts_with("%") {
            ids.append(&mut super::jobspec_to_array_poss(core, j));
        } else {
            for (i, job) in core.job_table.iter_mut().enumerate() {
                if job.pids[0].to_string() == *j {
                    ids.push(i);
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    let mut ans = -1;

    for _ in 0..ids.len() {
        let tmp = match ans {
            -1 => wait_next(core, &ids, var_name, f_opt)?,
            _ => wait_next(core, &ids, &None, f_opt)?,
        };

        if tmp.1 && ans == -1 {
            ans = tmp.0;
        }
    }
    Ok(ans)
}

fn wait_all(core: &mut ShellCore) -> Result<i32, ExecError> {
    let mut exit_status = 0;
    let mut remove_list = vec![];
    for pos in 0..core.job_table.len() {
        let result = core.job_table[pos].nonblock_wait(&core.sigint)?;
        exit_status = result.0;
        if result.1 { 
            remove_list.push(pos);
        }
    }

    for pos in remove_list.into_iter().rev() {
        super::remove(core, pos);
    }

    Ok(exit_status)
}

fn wait_arg_job(
    core: &mut ShellCore,
    com: &str,
    arg: &str,
    var_name: &Option<String>,
    f_opt: bool,
) -> Result<(i32, bool), ExecError> {
    if arg.starts_with("%") {
        return wait_jobspec(core, com, arg, var_name, f_opt);
    }

    if let Ok(pid) = arg.parse::<i32>() {
        return wait_pid(core, pid, var_name, f_opt);
    }

    Ok((127, false))
}

fn wait_jobspec(
    core: &mut ShellCore,
    com: &str,
    jobspec: &str,
    var_name: &Option<String>,
    f_opt: bool,
) -> Result<(i32, bool), ExecError> {
    match super::jobspec_to_array_pos(core, com, jobspec) {
        Some(pos) => wait_a_job(core, pos, var_name, f_opt),
        None => Ok((127, false)),
    }
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
        super::remove(core, drop);
    }
    Ok((exit_status, true))
}


fn wait_pid(core: &mut ShellCore, pid: i32, var_name: &Option<String>, f_opt: bool) -> Result<(i32, bool), ExecError> {
    match super::pid_to_array_pos(pid, &core.job_table) {
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
            builtins::error_(127, "wait", "invalpos jobpos", core),
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
        super::remove(core, pos);
        Ok(ans)
    }
}

