//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{exit, Feeder, Script, ShellCore, signal};
use crate::error;
use crate::error::exec::ExecError;
use nix::unistd;
use nix::errno::Errno;
use nix::sys::{resource, wait};
use nix::sys::resource::UsageWho;
use nix::sys::signal::Signal;
use nix::sys::wait::{WaitPidFlag, WaitStatus};
use nix::time::{clock_gettime, ClockId};
use nix::unistd::Pid;
use std::process;
use std::ffi::CString;
use std::sync::atomic::Ordering::Relaxed;

pub fn wait_pipeline(core: &mut ShellCore, pids: Vec<Option<Pid>>,
                     exclamation: bool, time: bool) -> Vec<WaitStatus> {
    if pids.len() == 1 && pids[0] == None {
        if time {
            show_time(core);
        }
        if exclamation {
            core.flip_exit_status();
        }
        exit::check_e_option(core);
        return vec![];
    }

    let mut pipestatus = vec![];
    let mut ans = vec![];
    for pid in &pids {
        let ws = wait_process(core, pid.expect("SUSHI INTERNAL ERROR (no pid)"));
        ans.push(ws);

        pipestatus.push(core.db.exit_status);
    }

    if time {
        show_time(core);
    }
    set_foreground(core);
    let _ = core.db.set_array("PIPESTATUS", pipestatus.iter().map(|e|e.to_string()).collect(), None);

    if core.options.query("pipefail") {
        pipestatus.retain(|e| *e != 0);

        if pipestatus.len() != 0 {
            core.db.exit_status = pipestatus[pipestatus.len()-1];
        }
    }

    if exclamation {
        core.flip_exit_status();
    }

    exit::check_e_option(core);

    ans
}

fn wait_process(core: &mut ShellCore, child: Pid) -> WaitStatus {
    let waitflags = match core.is_subshell {
        true  => None,
        false => Some(WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED)
    };

    let ws = wait::waitpid(child, waitflags);

    core.db.exit_status = match ws {
        Ok(WaitStatus::Exited(_pid, status)) => status,
        Ok(WaitStatus::Signaled(pid, signal, coredump)) => error::signaled(pid, signal, coredump),
        Ok(WaitStatus::Stopped(pid, signal)) => {
            eprintln!("Stopped Pid: {:?}, Signal: {:?}", pid, signal);
            148
        },
        Ok(unsupported) => {
            ExecError::UnsupportedWaitStatus(unsupported).print(core);
            1
        },
        Err(err) => {
            let msg = format!("Error: {:?}", err);
            exit::internal(&msg);
        },
    };

    if core.db.exit_status == 130 {
        core.sigint.store(true, Relaxed);
    }
    ws.expect("SUSH INTERNAL ERROR: no wait status")
}

pub fn set_foreground(core: &ShellCore) {
    let fd = match core.tty_fd.as_ref() {
        Some(fd) => fd,
        _        => return,
    };

    let pgid = unistd::getpgid(Some(Pid::from_raw(0)))
               .expect(&error::internal("cannot get pgid"));

    if unistd::tcgetpgrp(fd) == Ok(pgid) {
        return;
    }

    signal::ignore(Signal::SIGTTOU); //SIGTTOUを無視
    unistd::tcsetpgrp(fd, pgid)
        .expect(&error::internal("cannot get the terminal"));
    signal::restore(Signal::SIGTTOU); //SIGTTOUを受け付け
}

pub fn set_pgid(core :&ShellCore, pid: Pid, pgid: Pid) {
    let _ = unistd::setpgid(pid, pgid);
    if pid.as_raw() == 0 && pgid.as_raw() == 0 { //以下3行追加
        set_foreground(core);
    }
}

fn show_time(core: &ShellCore) {
     let real_end_time = clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();

     let core_usage = resource::getrusage(UsageWho::RUSAGE_SELF).unwrap();
     let children_usage = resource::getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

     let real_diff = real_end_time - core.measured_time.real;
     eprintln!("\nreal\t{}m{}.{:06}s", real_diff.tv_sec()/60,
               real_diff.tv_sec()%60, real_diff.tv_nsec()/1000);
     let user_diff = core_usage.user_time() + children_usage.user_time() - core.measured_time.user;
     eprintln!("user\t{}m{}.{:06}s", user_diff.tv_sec()/60,
               user_diff.tv_sec()%60, user_diff.tv_usec());
     let sys_diff = core_usage.system_time() + children_usage.system_time() - core.measured_time.sys;
     eprintln!("sys \t{}m{}.{:06}s", sys_diff.tv_sec()/60,
               sys_diff.tv_sec()%60, sys_diff.tv_usec());
}

pub fn exec_command(args: &Vec<String>, core: &mut ShellCore, path_search: bool) -> ! {
    let cargs = to_cargs(args);

    let result = match path_search {
        true  => unistd::execvp(&cargs[0], &cargs),
        false => unistd::execv(&cargs[0], &cargs),
    };

    match result {
        Err(Errno::E2BIG) => exit::arg_list_too_long(&args[0], core),
        Err(Errno::EACCES) => exit::permission_denied(&args[0], core),
        Err(Errno::ENOENT) => run_command_not_found(&args[0], core),
        Err(err) => {
            eprintln!("Failed to execute. {:?}", err);
            process::exit(127)
        }
        _ => exit::internal("never come here")
    }
}

fn run_command_not_found(arg: &String, core: &mut ShellCore) -> ! {
    if core.db.functions.contains_key("command_not_found_handle") {
        let s = "command_not_found_handle ".to_owned() + &arg.clone();
        let mut f = Feeder::new(&s);
        match Script::parse(&mut f, core, false) {
            Ok(Some(mut script)) => {let _ = script.exec(core);},
            Err(e) => e.print(core),
            _ => {},
        }
    }
    exit::not_found(&arg, core)
}

fn to_cargs(args: &Vec<String>) -> Vec<CString> {
    args.iter()
        .map(|a| CString::new(a.to_string()).unwrap())
        .collect()
}
