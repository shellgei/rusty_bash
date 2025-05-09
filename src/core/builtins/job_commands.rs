//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::{signal, utils};
use crate::core::JobEntry;
use crate::utils::arg;
use nix::sys::signal::Signal;
use nix::unistd;
use nix::unistd::Pid;
use std::{thread, time};

fn pid_to_array_pos(pid: i32, jobs: &Vec<JobEntry>) -> Option<usize> {
    for i in 0..jobs.len() {
        if jobs[i].pids[0].as_raw() == pid {
            return Some(i);
        }
    }
    None
}

fn jobid_to_jobentry(id: usize, jobs: &mut Vec<JobEntry>) -> Option<&mut JobEntry> {
    for job in jobs.iter_mut() {
        if job.id == id {
            return Some(job);
        }
    }
    None
}

pub fn bg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if core.job_table.is_empty() {
        return 1;
    }

    let mut args = arg::dissolve_options(args);
    if ! core.db.flags.contains('m') {
        return super::error_exit(1, &args[0], "no job control", core);
    }

    if arg::consume_option("-s", &mut args) {
        return super::error_exit(1, &args[0], "-s: invalid option", core);
    }

    if args.len() == 1 {
        let id = core.job_table_priority[0];
        match jobid_to_jobentry(id, &mut core.job_table) {
            Some(job) => job.send_cont(),
            _ => return 1, 
        }
    }else if args.len() == 2 {
        match jobspec_to_array_pos(core, &args[1]) {
            Some(pos) => core.job_table[pos].send_cont(),
            None      => return 1,
        }
    }

    0
}

pub fn fg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);
    if ! core.db.flags.contains('m') {
        return super::error_exit(1, &args[0], "no job control", core);
    }

    if arg::consume_option("-s", &mut args) {
        return super::error_exit(1, &args[0], "-s: invalid option", core);
    }

    let id = if args.len() == 1 {
        if core.job_table_priority.is_empty() {
            return 1;
        }
        core.job_table_priority[0]
    }else if args.len() == 2 {
        match jobspec_to_array_pos(core, &args[1]) {
            Some(pos) => core.job_table[pos].id,
            None => return 1,
        }
    }else{
        return 1;
    };

    let fd = match core.tty_fd.as_ref() {
        Some(fd) => fd,
        _        => return 1,
    };


    let job = match jobid_to_jobentry(id, &mut core.job_table) {
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
        println!("{}", &job.text);
        job.send_cont();
        exit_status = job.update_status(true, false).unwrap_or(1);

        if let Ok(mypgid) = unistd::getpgid(Some(Pid::from_raw(0))) {
            let _ = unistd::tcsetpgrp(fd, mypgid);
        }
    }

    signal::restore(Signal::SIGTTOU);
    exit_status
}

fn jobspec_to_array_pos(core: &mut ShellCore, jobspec: &str) -> Option<usize> {
    let poss = jobspec_to_array_poss(core, jobspec);
    if poss.is_empty() {
        let msg = format!("{}: no such job", &jobspec);
        super::error_exit(127, "jobs", &msg, core);
        return None;
    }else if poss.len() > 1 {
        let msg = format!("{}: ambiguous job spec", &jobspec[1..]);
        super::error_exit(127, "jobs", &msg, core);
        return None;
    }

    Some(poss[0])
}

fn jobspec_to_array_poss(core: &mut ShellCore, jobspec: &str) -> Vec<usize> {
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

    let ids = jobspec_to_array_poss(core, &jobspec);

    if ids.is_empty() {
        let msg = format!("{}: no such job", &jobspec);
        return super::error_exit(127, "jobs", &msg, core);
    }
    if ids.len() > 1 && ! jobspec.is_empty() {
        let msg = format!("{}: ambiguous job spec", &jobspec[1..]);
        super::error_exit(127, "jobs", &msg, core);
        let msg = format!("{}: no such job", &jobspec);
        return super::error_exit(127, "jobs", &msg, core);
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
    let mut remove = vec![];
    for id in ids {
        if core.job_table[id].print(&core.job_table_priority, l_opt, r_opt, s_opt, true) {
            remove.push(id);
        }
    }

    for id in remove.into_iter().rev() {
        core.job_table.remove(id);
    }

    0
}

fn wait_jobspec(core: &mut ShellCore, jobspec: &str,
                var_name: &Option<String>, f_opt: bool) -> (i32, bool) {
    match jobspec_to_array_pos(core, jobspec) {
        Some(pos) => wait_a_job(core, pos, var_name, f_opt),
        None => return (127, false),
    }
}

fn wait_next(core: &mut ShellCore, ids: &Vec<usize>,
             var_name: &Option<String>, f_opt: bool) -> (i32, bool) {
    if core.job_table_priority.is_empty() {
        return (127, false);
    }

    let mut exit_status = 0;
    let mut drop = 0;
    let mut end = false;
    let mut pid = String::new();
    loop {
        thread::sleep(time::Duration::from_millis(10)); //0.1秒周期に変更
        for (i, job) in core.job_table.iter_mut().enumerate() {
            if ! ids.contains(&i) && ! ids.is_empty() {
                continue;
            }

            if let Ok(es) = job.update_status(false, true) {
                if job.display_status == "Done"
                || (job.display_status == "Stopped" && ! f_opt) {
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
    (exit_status, true)
}

fn wait_pid(core: &mut ShellCore, pid: i32,
            var_name: &Option<String>, f_opt: bool) -> (i32, bool) {
    match pid_to_array_pos(pid, &core.job_table) {
        Some(i) => wait_a_job(core, i, var_name, f_opt),
        None => (1, false),
    }
}

fn wait_a_job(core: &mut ShellCore, pos: usize,
              var_name: &Option<String>, f_opt: bool) -> (i32, bool) {
    if core.job_table.len() < pos {
        return (super::error_exit(127, "wait", "invalpos jobpos", core), false);
    }

    let pid = core.job_table[pos].pids[0].to_string();

    let ans = match core.job_table[pos].update_status(true, false) {
        Ok(n) => {
            if let Some(var) = var_name {
                core.db.unset(&var);
                if let Err(e) = core.db.set_param(&var, &pid, None) {
                    e.print(core);
                }
            }
            (n, true)
        },
        Err(e) => { e.print(core); return (1, false) },
    };

    if f_opt && core.job_table[pos].display_status == "Stopped" {
        wait_a_job(core, pos, var_name, f_opt)
    }else{
        core.job_table.remove(pos);
        ans
    }
}

fn wait_arg_job(core: &mut ShellCore, arg: &String,
                var_name: &Option<String>, f_opt: bool) -> (i32, bool) {
    if arg.starts_with("%") {
        return wait_jobspec(core, &arg, &var_name, f_opt);
    }

    if let Ok(pid) = arg.parse::<i32>() {
        return wait_pid(core, pid, &var_name, f_opt);
    }

    (1, false) 
}

pub fn wait(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if core.is_subshell {
        super::error_exit(127, &args[0], "called from subshell", core);
    }

    let mut args = arg::dissolve_options(args);
    let var_name = arg::consume_with_next_arg("-p", &mut args);
    let f_opt = arg::consume_option("-f", &mut args);

    if args.len() <= 1 {
        let mut exit_status = 0;
        let mut remove_list = vec![];
        for pos in 0..core.job_table.len() {
            match core.job_table[pos].update_status(true, false) {
                Ok(n) => {
                    //if core.job_table[pos].print(&core.job_table_priority, false, false, false, false) {
                    if core.job_table[pos].display_status == "Done" {
                        remove_list.push(pos);
                    }
                    exit_status = n;
                },
                Err(e) => {
                    e.print(core);
                    exit_status = 1;
                    break;
                },
            }
        }

        for pos in remove_list.into_iter().rev() {
            core.job_table.remove(pos);
        }

        return exit_status;
    }

    if args[1] == "-n" {
        let mut jobs = arg::consume_with_subsequents("-n", &mut args);
        jobs.remove(0);
        if jobs.is_empty() {
            return wait_next(core, &vec![], &var_name, f_opt).0;
        }

        let mut ids = vec![];
        for j in &jobs {
            if j.starts_with("%") {
                ids.append(&mut jobspec_to_array_poss(core, &j));
            }else{
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
                -1 => wait_next(core, &ids, &var_name, f_opt),
                _  => wait_next(core, &ids, &None, f_opt),
            };

            if tmp.1 == true && ans == -1 {
                ans = tmp.0;
            }
        }
        return ans;
    }

    wait_arg_job(core, &args[1], &var_name, f_opt).0
}

/* TODO: implement original kill */
pub fn kill(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let path = utils::get_command_path(&args[0], core);

    match path.is_empty() {
        true  => return 1,
        false => args[0] = path,
    }

    if args.len() >= 3 && args[2].starts_with("%") {
        match jobspec_to_array_pos(core, &args[2]) {
            Some(id) => args[2] = core.job_table[id].pids[0].to_string(),
            None => return 1,
        }
    }

    args.insert(0, "eval".to_string());
    super::eval(core, args)
}

pub fn disown(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let args = arg::dissolve_options(args);

    if args.len() == 1 {
        let ids = jobspec_to_array_poss(core, "%%");

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
