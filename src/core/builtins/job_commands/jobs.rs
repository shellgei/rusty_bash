//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::builtins;
use crate::utils::arg;
use crate::ShellCore;

pub fn jobs(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = arg::dissolve_options(args);
    if arg::consume_arg("-n", &mut args) {
        core.jobtable_print_status_change();
        return 0;
    }

    let jobspecs = arg::consume_starts_with("%", &mut args);
    let jobspec = match jobspecs.last() {
        Some(s) => s.clone(),
        None => String::new(),
    };

    if core.job_table.is_empty() && jobspec.is_empty() {
        return 0;
    }

    let poss = super::jobspec_to_array_poss(core, &jobspec);

    if poss.is_empty() {
        let msg = format!("{}: no such job", &jobspec);
        return builtins::error_(127, "jobs", &msg, core);
    }
    if poss.len() > 1 && !jobspec.is_empty() {
        let msg = format!(
            "{}: ambiguous job spec",
            jobspec.strip_prefix('%').unwrap_or(&jobspec)
        );
        builtins::error_(127, "jobs", &msg, core);
        let msg = format!("{}: no such job", &jobspec);
        return builtins::error_(127, "jobs", &msg, core);
    }

    if arg::consume_arg("-p", &mut args) {
        for id in poss {
            core.job_table[id].print_p();
        }
        return 0;
    }

    if !jobspec.is_empty() {
        let l_opt = arg::consume_arg("-l", &mut args);
        let r_opt = arg::consume_arg("-r", &mut args);
        let s_opt = arg::consume_arg("-s", &mut args);
        if core.job_table[poss[0]].print(&core.job_table_priority, l_opt, r_opt, s_opt, true) {
            super::remove(core, poss[0]);
        }
        return 0;
    }

    print(core, &args);
    0
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
        super::remove(core, pos);
    }
}
