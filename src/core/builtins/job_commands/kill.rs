//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, utils};
use crate::builtins::job_commands;
use crate::core::builtins;

/* TODO: implement original kill */
pub fn kill(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = args.to_owned();
    //let mut args = arg::dissolve_options(args);
    let path = utils::get_command_path(&args[0], core);

    match path.is_empty() {
        true => return 1,
        false => args[0] = path,
    }

    if args.len() >= 3 && args[2].starts_with("%") {
        match job_commands::jobspec_to_array_pos(core, &args[0], &args[2]) {
            Some(id) => args[2] = core.job_table[id].pids[0].to_string(),
            None => return 1,
        }
    }

    let com = args[0].to_string();
    for arg in args.iter_mut() {
        if arg == "-n" {
            *arg = "-s".to_string();
        }
        if arg.starts_with("%") {
            if let Some(pos) = job_commands::jobspec_to_array_pos(core, &com, arg) {
                *arg = core.job_table[pos].pids[0].to_string();
            } else {
                let msg = format!("{}: no such job", &arg);
                return builtins::error_(127, "jobs", &msg, core);
            }
        }
    }

    builtins::run_external(core, &args, |es| es > 0)
}
