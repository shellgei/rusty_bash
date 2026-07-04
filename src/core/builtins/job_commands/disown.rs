//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::builtins;
use crate::utils::arg;

pub fn disown(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = arg::dissolve_options(args);
    let h_opt = arg::consume_arg("-h", &mut args);
    let _r_opt = arg::consume_arg("-r", &mut args); //TODO: implement

    if args.len() == 1 {
        let ids = super::jobspec_to_array_poss(core, "%%");

        if ids.len() == 1 {
            super::remove_coproc(core, ids[0]);
            core.job_table.remove(ids[0]);
            core.job_table_priority.remove(0);
            return 0;
        }

        return 1;
    }

    if args.len() == 2 && args[1] == "-a" {
        core.job_table.clear();
        core.job_table_priority.clear();
        return 0;
    }

    for a in &args[1..] {
        if a.starts_with("-") {
            let msg = format!("{}: invalid option", &a);
            builtins::error_(127, &args[0], &msg, core);
            eprintln!("disown: usage: disown [-h] [-ar] [jobspec ... | pid ...]");
            return 127;
        }
    }

    for a in &args[1..] {
        if let Some(pos) = super::jobspec_to_array_pos(core, &args[0], a) {
            if h_opt {
                //TODO: to make each job doesn't stop by SIGHUP
            } else {
                super::remove(core, pos);
            }
        }
    }

    0
}
