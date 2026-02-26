//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::builtins;
use crate::utils::arg;
use crate::ShellCore;
use crate::signal;
use nix::sys::signal::Signal;
use nix::unistd;
use nix::unistd::Pid;

pub fn fg(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    let mut args = arg::dissolve_options(&args);
    if !core.db.flags.contains('m') {
        return builtins::error_(1, &args[0], "no job control", core);
    }

    if arg::consume_arg("-s", &mut args) {
        return builtins::error_(1, &args[0], "-s: invalid option", core);
    }

    let id = if args.len() == 1 {
        if core.job_table_priority.is_empty() {
            return 1;
        }
        core.job_table_priority[0]
    } else if args.len() == 2 {
        match super::jobspec_to_array_pos(core, &args[0], &args[1]) {
            Some(pos) => core.job_table[pos].id,
            None => return 1,
        }
    } else {
        return 1;
    };

    let pos = match super::jobid_to_pos(id, &mut core.job_table) {
        Some(i) => i,
        _ => return 1,
    };

    if core.job_table[pos].no_control {
        let id = core.job_table[pos].id;
        let msg = format!("job {} started without job control", &id);
        return builtins::error_(1, &args[0], &msg, core);
    }

    let pgid = core.job_table[pos].solve_pgid();
    if pgid.as_raw() == 0 {
        return 1;
    }

    signal::ignore(Signal::SIGTTOU);

    let mut exit_status = 1;
    if let Some(fd) = core.tty_fd.as_ref() {
        if core.fds.tcsetpgrp(*fd, pgid).is_ok() {
            println!("{}", &core.job_table[pos].text);
            core.job_table[pos].send_cont();
            exit_status = core.job_table[pos].update_status(true, false).unwrap_or(1);

            if let Ok(mypgid) = unistd::getpgid(Some(Pid::from_raw(0))) {
                let _ = core.fds.tcsetpgrp(*fd, mypgid);
            }
        }
    } else {
        println!("{}", &core.job_table[pos].text);
        core.job_table[pos].send_cont();
        exit_status = core.job_table[pos].update_status(true, false).unwrap_or(1);
    }

    super::remove(core, pos);
    signal::restore(Signal::SIGTTOU);
    exit_status
}
