//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::JobEntry;
use crate::signal;
use crate::error;
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

fn arg_to_id(s: &str, priority: &Vec<usize>, table: &Vec<JobEntry>) -> Result<usize, String> {
    if s == "%+" {
        return match priority.len() {
            0 => Err("%+: no such job".to_string()), 
            _ => Ok(priority[0]),
        };
    }

    if s == "%-" {
        return match priority.len() {
            0 => Err("%-: no such job".to_string()), 
            1 => Err("%-: no such job".to_string()), 
            _ => Ok(priority[1]),
        };
    }

    let word = &s[1..];
    let mut ans = 0;
    for job in table {
        let jobname = job.text.split(" ").nth(0).unwrap();
        if jobname == word {
            if ans != 0 {
                return Err((s.to_owned() + ": ambiguous job spec").to_string());
            }
            ans = job.id;
        }
    }

    if ans != 0 {
        return Ok(ans);
    }

    if s.starts_with("%") {
        return match word.parse::<usize>() {
            Ok(n)  => Ok(n),
            Err(_) => Err((s.to_owned() + ": no such job").to_string()),
        };
    }

    Err((s.to_owned() + ": no such job").to_string())
}

pub fn bg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let id = if args.len() == 1 {
        if core.job_table_priority.is_empty() {
            return 1;
        }
        core.job_table_priority[0]
    }else if args.len() == 2 {
        match arg_to_id(&args[1], &core.job_table_priority, &core.job_table) {
            Ok(n) => n,
            Err(s) => {
                error::print(&("bg: ".to_owned() + &s), core);
                return 1;
            },
        }
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
        if core.job_table_priority.is_empty() {
            return 1;
        }
        core.job_table_priority[0]
    }else if args.len() == 2 {
        match arg_to_id(&args[1], &core.job_table_priority, &core.job_table) {
            Ok(n) => n,
            Err(s) => {
                error::print(&s, core);
                return 1;
            },
        }
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

    signal::ignore(Signal::SIGTTOU);

    let mut exit_status = 1;
    if let Ok(_) =  unistd::tcsetpgrp(fd, pgid) {
        eprintln!("{}", &job.text);
        job.send_cont();
        exit_status = job.update_status(true);

        if let Ok(mypgid) = unistd::getpgid(Some(Pid::from_raw(0))) {
            let _ = unistd::tcsetpgrp(fd, mypgid);
        }
    }

    signal::restore(Signal::SIGTTOU);
    exit_status
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

    let id = match arg_to_id(&args[1], &core.job_table_priority, &core.job_table) {
        Ok(n)  => n,
        Err(s) => {
            error::print(&("wait: ".to_owned() + &s), core);
            return 1;
        },
    };
    match id_to_job(id, &mut core.job_table) {
        Some(job) => {job.update_status(true);},
        _ => return 1, 
    }

    0
}
