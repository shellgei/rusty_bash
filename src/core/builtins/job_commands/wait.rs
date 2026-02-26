//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::builtins;
use crate::error::exec::ExecError;
use crate::utils::arg;

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
        return Ok(super::wait_next(core, &[], var_name, f_opt)?.0);
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
            -1 => super::wait_next(core, &ids, var_name, f_opt)?,
            _ => super::wait_next(core, &ids, &None, f_opt)?,
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
        return super::wait_pid(core, pid, var_name, f_opt);
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
        Some(pos) => super::wait_a_job(core, pos, var_name, f_opt),
        None => Ok((127, false)),
    }
}
