//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::builtins;
use crate::utils::arg;
use crate::ShellCore;

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
        match super::wait_n(core, &mut args, &var_name, f_opt) {
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

