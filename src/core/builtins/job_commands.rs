//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::JobEntry;
use crate::core::{ignore_signal, restore_signal};
use nix::sys::signal::Signal;
use nix::unistd;
use nix::unistd::Pid;

fn id_to_job(id: usize, jobs: &mut Vec<JobEntry>) -> Option<&mut JobEntry> {
    for job in jobs.iter_mut() {
        if job.id == id {
            return Some(job);
        }
    }

    None
}

fn arg_to_id(s: &str, priority: &Vec<usize>) -> usize {
    if s == "%+" {
        return match priority.len() {
            0 => 0, 
            _ => priority[0],
        };
    }

    if s == "%-" {
        return match priority.len() {
            0 => 0, 
            1 => 0, 
            _ => priority[1],
        };
    }

    if s.starts_with("%") {
        return s[1..].parse::<usize>().unwrap_or(0);
    }

    0
}

pub fn bg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let id = if args.len() == 1 {
        if core.job_table_priority.len() == 0 {
            return 1;
        }
        core.job_table_priority[0]
    }else if args.len() == 2 {
        arg_to_id(&args[1], &core.job_table_priority)
    }else{
        return 1;
    };

    match id_to_job(id, &mut core.job_table) {
        Some(job) => job.send_cont(),
        _ => return 1, 
    }
    0
}

pub fn fg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let fd = match core.tty_fd.as_ref() {
        Some(fd) => fd,
        _        => return 1,
    };

    let id = if args.len() == 1 {
        if core.job_table_priority.len() == 0 {
            return 1;
        }
        core.job_table_priority[0]
    }else if args.len() == 2 {
        arg_to_id(&args[1], &core.job_table_priority)
    }else{
        return 1;
    };

    let job = match id_to_job(id, &mut core.job_table) {
        Some(job) => job,
        _ => return 1, 
    };

    let pgid = job.solve_pgid();
    if pgid.as_raw() == 0 {
        return 1;
    }

    ignore_signal(Signal::SIGTTOU);
    match unistd::tcsetpgrp(fd, pgid) {
        Ok(_) => {
            eprintln!("{}", &job.text);
            job.send_cont();
            job.update_status(false);
            job.update_status(true);
            let mypgid = unistd::getpgid(Some(Pid::from_raw(0)))
                   .expect("sush(fatal): cannot get pgid");
            let _ = unistd::tcsetpgrp(fd, mypgid);
        },
        _ => {
            restore_signal(Signal::SIGTTOU);
            return 1;
        },
    }
    restore_signal(Signal::SIGTTOU);
    0
}

pub fn jobs(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    for job in core.job_table.iter() {
        job.print(&core.job_table_priority);
    }
    0
}

pub fn wait(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        for job in core.job_table.iter_mut() {
            job.update_status(true);
        }
        return 0;
    }

    let id = arg_to_id(&args[1], &core.job_table_priority);
    match id_to_job(id, &mut core.job_table) {
        Some(job) => job.update_status(true),
        _ => return 1, 
    }

    0
}
