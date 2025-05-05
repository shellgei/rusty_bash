//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::{error, signal, utils};
use crate::core::JobEntry;
use crate::utils::arg;
use nix::sys::signal::Signal;
use nix::unistd;
use nix::unistd::Pid;
use std::{thread, time};

fn pid_to_jobid(pid: i32, jobs: &Vec<JobEntry>) -> Option<usize> {
    for i in 0..jobs.len() {
        if jobs[i].pids[0].as_raw() == pid {
            return Some(i);
        }
    }
    None
}

fn id_to_job(id: usize, jobs: &mut Vec<JobEntry>) -> Option<&mut JobEntry> {
    for job in jobs.iter_mut() {
        if job.id == id {
            return Some(job);
        }
    }
    None
}

fn job_to_id(s: &str, priority: &Vec<usize>, table: &Vec<JobEntry>) -> Result<usize, String> {
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
        match job_to_id(&args[1], &core.job_table_priority, &core.job_table) {
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
        match job_to_id(&args[1], &core.job_table_priority, &core.job_table) {
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
        exit_status = job.update_status(true, false).unwrap_or(1);

        if let Ok(mypgid) = unistd::getpgid(Some(Pid::from_raw(0))) {
            let _ = unistd::tcsetpgrp(fd, mypgid);
        }
    }

    signal::restore(Signal::SIGTTOU);
    exit_status
}

fn jobspec_choice(core: &mut ShellCore, jobspec: &str) -> Vec<usize> {
    if jobspec == "" {
        return (0..core.job_table.len()).collect();
    }

    if core.job_table.is_empty() {
        return vec![];
    }

    let s = &jobspec[1..];
    let mut ans = vec![];

    if let Ok(n) = s.parse::<usize>() {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if n == job.id {
                ans.push(i);
            }
        }
    }else if s == "%" || s == "+" {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if job.id == core.job_table_priority[0] {
                ans.push(i);
            }
        }
    }else if s == "-" {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if core.job_table_priority.len() < 2 {
                if job.id == core.job_table_priority[0] {
                    ans.push(i);
                }
            }else {
                if job.id == core.job_table_priority[1] {
                    ans.push(i);
                }
            }
        }
    }else if s.starts_with("?") {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if job.text.contains(&s[1..]) {
                ans.push(i);
            }
        }
    }else {
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if job.text.starts_with(s) {
                ans.push(i);
            }
        }
    }

    ans
}

pub fn jobs(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);
    if arg::consume_option("-n", &mut args) {
        core.jobtable_print_status_change();
        return 0;
    }

    let jobspecs = arg::consume_starts_with("%", &mut args);
    let jobspec = match jobspecs.last() {
        Some(s) => s.clone(),
        None => String::new(),
    };

    if core.job_table.is_empty() && jobspec == "" {
        return 0;
    }

    let ids = jobspec_choice(core, &jobspec);

    if ids.is_empty() {
        let msg = format!("{}: no such job", &jobspec);
        return super::error_exit(1, "jobs", &msg, core);
    }
    if ids.len() > 1 && ! jobspec.is_empty() {
        let msg = format!("{}: ambiguous job spec", &jobspec[1..]);
        super::error_exit(1, "jobs", &msg, core);
        let msg = format!("{}: no such job", &jobspec);
        return super::error_exit(1, "jobs", &msg, core);
    }

    if arg::consume_option("-p", &mut args) {
        for id in ids {
            core.job_table[id].print_p();
        }
        return 0;
    }

    let l_opt = arg::consume_option("-l", &mut args);
    let r_opt = arg::consume_option("-r", &mut args);
    let s_opt = arg::consume_option("-s", &mut args);
    for id in ids {
        core.job_table[id].print(&core.job_table_priority, l_opt, r_opt, s_opt, true);
    }

    0
}

fn wait_jobspec(core: &mut ShellCore, jobspec: &str, var_name: &Option<String>) -> i32 {
    let ids = jobspec_choice(core, jobspec);
    if ids.is_empty() {
        let msg = format!("{}: no such job", &jobspec);
        return super::error_exit(1, "jobs", &msg, core);
    }
    if ids.len() > 1 {
        let msg = format!("{}: ambiguous job spec", &jobspec[1..]);
        return super::error_exit(1, "jobs", &msg, core);
    }

    wait_a_job(core, ids[0], var_name)
}

fn wait_next(core: &mut ShellCore, var_name: &Option<String>) -> i32 {
    if core.job_table_priority.is_empty() {
        return 127;
    }

    let mut exit_status = 0;
    let mut drop = 0;
    let mut end = false;
    let mut pid = String::new();
    loop {
        thread::sleep(time::Duration::from_millis(10)); //0.1秒周期に変更
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if let Ok(es) = job.update_status(false, true) {
                if job.display_status == "Done"
                || job.display_status == "Stopped" {
                    exit_status = es;
                    drop = i;
                    end = true;
                    pid = job.pids[0].to_string();
                    break;
                }
            }
        }

        if end {
            break;
        }
    }

    if let Some(var) = var_name {
        core.db.unset(&var);
        if let Err(e) = core.db.set_param(&var, &pid, None) {
            e.print(core);
        }
    }

    let job_id = core.job_table[drop].id;
    core.job_table.remove(drop);
    core.job_table_priority.retain(|id| *id != job_id);
    exit_status
}

fn wait_pid(core: &mut ShellCore, pid: i32) -> i32 {
    match pid_to_jobid(pid, &core.job_table) {
        Some(i) => wait_a_job(core, i, &None),
        None => 1,
    }
}

fn wait_a_job(core: &mut ShellCore, id: usize, var_name: &Option<String>) -> i32 {
    if core.job_table.len() < id {
        return super::error_exit(1, "wait", "invalid jobid", core);
    }

    let pid = core.job_table[id].pids[0].to_string();

    match core.job_table[id].update_status(true, false) {
        Ok(n) => {
            if let Some(var) = var_name {
                core.db.unset(&var);
                if let Err(e) = core.db.set_param(&var, &pid, None) {
                    e.print(core);
                }
            }
            n
        },
        Err(e) => { e.print(core); 1 },
    }
}

pub fn wait(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);
    let var_name = arg::consume_with_next_arg("-p", &mut args);

    if args.len() <= 1 {
        let mut exit_status = 0;
        for job in core.job_table.iter_mut() {
            match job.update_status(true, false) {
                Ok(n) => exit_status = n,
                Err(e) => {
                    e.print(core);
                    return 1;
                },
            }
        }
        return exit_status;
    }

    if args[1] == "-n" {
        let mut jobs = arg::consume_with_subsequents("-n", &mut args);
        jobs.remove(0);
        if jobs.is_empty() {
            return wait_next(core, &var_name);
        }

        let first = jobs.remove(0);
        let ans = wait_jobspec(core, &first, &var_name);

        for j in jobs {
            let _ = wait_jobspec(core, &j, &None);
        }
        return ans;
    }

    if args[1].starts_with("%") {
        return wait_jobspec(core, &args[1], &var_name);
    }

    if let Ok(pid) = args[1].parse::<i32>() {
        return wait_pid(core, pid);
    }

    /*
    let id = match job_to_id(&args[1], &core.job_table_priority, &core.job_table) {
        Ok(n)  => n,
        Err(s) => {
            error::print(&("wait: ".to_owned() + &s), core);
            return 1;
        },
    };

    match id_to_job(id, &mut core.job_table) {
        Some(job) => {
            return match job.update_status(true, false) {
                Ok(n) => n,
                Err(e) => { e.print(core); 1 },
            };
        }
        _ => return 1, 
    }
    */
    1
}

/* TODO: implement original kill */
pub fn kill(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let path = utils::get_command_path(&args[0], core);

    match path.is_empty() {
        true  => return 1,
        false => args[0] = path,
    }

    if args.len() >= 3 && args[2].starts_with("%") {
        let ids = jobspec_choice(core, &args[2]);

        if ids.is_empty() {
            let msg = format!("{}: no such job", &args[2]);
            return super::error_exit(1, "jobs", &msg, core);
        }
        if ids.len() > 1 {
            let msg = format!("{}: ambiguous job spec", &args[2][1..]);
            return super::error_exit(1, "jobs", &msg, core);
        }

        args[2] = core.job_table[ids[0]].pids[0].to_string();
    }

    args.insert(0, "eval".to_string());
    super::eval(core, args)
}

pub fn disown(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let args = arg::dissolve_options(args);

    if args.len() == 1 {
        let ids = jobspec_choice(core, "%%");

        if ids.len() == 1 {
            core.job_table.remove(ids[0]);
            core.job_table_priority.remove(0);
        }

        return 1;
    }

    if args.len() == 2 || args[1] == "-a" {
        core.job_table.clear();
        core.job_table_priority.clear();
    }

    0
}
