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
        match super::wait_all(core) {
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
        match super::wait_arg_job(core, &args[0], &args[1], &var_name, f_opt) {
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
