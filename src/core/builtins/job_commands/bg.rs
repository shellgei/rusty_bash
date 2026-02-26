//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::builtins;
use crate::utils::arg;
use crate::ShellCore;

pub fn bg(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    if core.job_table.is_empty() {
        return 1;
    }

    let mut args = arg::dissolve_options(&args);
    if !core.db.flags.contains('m') {
        return builtins::error_(1, &args[0], "no job control", core);
    }

    if arg::consume_arg("-s", &mut args) {
        return builtins::error_(1, &args[0], "-s: invalid option", core);
    }

    let pos = match args.len() {
        1 => {
            let id = core.job_table_priority[0];
            super::jobid_to_pos(id, &mut core.job_table)
        }
        2 => super::jobspec_to_array_pos(core, &args[0], &args[1]),
        _ => None,
    };

    match pos {
        Some(p) => {
            let id = core.job_table[p].id;

            if core.job_table[p].no_control {
                let msg = format!("job {} started without job control", &id);
                return builtins::error_(1, &args[0], &msg, core);
            }

            if core.job_table[p].display_status == "Running" {
                let msg = format!("job {} already in background", &id);
                return builtins::error_(0, &args[0], &msg, core);
            }

            let priority = super::get_priority(core, p);
            let symbol = match priority {
                0 => "+",
                1 => "-",
                _ => " ",
            };
            println!("[{}]{} {} &", &id, &symbol, &core.job_table[p].text);
            core.job_table[p].send_cont()
        }
        None => return 1,
    }
    0
}
