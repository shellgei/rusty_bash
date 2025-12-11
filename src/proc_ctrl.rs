//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, signal};
use crate::error::exec::ExecError;
use crate::utils::c_string;
use nix::unistd;
use nix::errno::Errno;
use nix::sys::wait;
use nix::sys::wait::{WaitStatus, WaitPidFlag};
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use std::process;
use std::sync::atomic::Ordering::Relaxed;

pub fn wait_pipeline(
    core: &mut ShellCore,
    pids: Vec<Option<Pid>>,
) -> Vec<WaitStatus> {
    if pids.len() == 1 && pids[0].is_none() {
        return vec![];
    }

    let last_exit_status = core.db.get_param("?").unwrap();
    let mut ans = vec![];
    for pid in &pids {
        if pid.is_some() {
            //None: lastpipe
            let ws = wait_process(core, pid.unwrap());
            ans.push(ws);
        } else {
            let _ = core.db.set_param("?", &last_exit_status.to_string(), None);
        }
    }

    let _ = set_foreground(core);

    ans
}

fn wait_process(core: &mut ShellCore, child: Pid) -> WaitStatus {
    let waitflags = match core.is_subshell {
        true => None,
        false => Some(WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED),
    };

    let ws = wait::waitpid(child, waitflags);

    let exit_status = match ws {
        Ok(WaitStatus::Exited(_pid, status)) => status,
        Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
            eprintln!("Pid: {pid:?}, Signal: {signal:?}");
            128+signal as i32
        },
        Ok(WaitStatus::Stopped(pid, signal)) => {
            eprintln!("Stopped Pid: {pid:?}, Signal: {signal:?}");
            148
        }
        Ok(unsupported) => {
            ExecError::UnsupportedWaitStatus(unsupported).print(core);
            1
        }
        Err(err) => {
            panic!("Error: {err:?}");
        }
    };

    if exit_status == 130 {
        core.sigint.store(true, Relaxed);
    }
    let _ = core.db.set_param("?", &exit_status.to_string(), None);
    ws.expect("SUSH INTERNAL ERROR: no wait status")
}

fn set_foreground(core: &mut ShellCore) -> Result<(), ExecError> {
    let fd = match core.tty_fd.as_ref() {
        Some(fd) => fd,
        _ => return Ok(()),
    };

    let pgid = unistd::getpgid(Some(Pid::from_raw(0)))
               .expect("sush(fatal): cannot get pgid");

    if let Ok(n) = core.fds.tcgetpgrp(*fd) {
        if n == pgid { 
            return Ok(());
        }       
    }

    signal::ignore(Signal::SIGTTOU); //SIGTTOUを無視
    core.fds.tcsetpgrp(*fd, pgid)
        .expect("sush(fatal): cannot get the terminal");
    signal::restore(Signal::SIGTTOU); //SIGTTOUを受け付け
    Ok(())
}

pub fn set_pgid(core: &mut ShellCore, pid: Pid, pgid: Pid) {
    let _ = unistd::setpgid(pid, pgid);

    if pid.as_raw() == 0 && pgid.as_raw() == 0 {
        let _ = set_foreground(core);
    }
}

pub fn exec_command(args: &[String]) -> ! {
    let cargs = c_string::to_cargs(args);
    let result = unistd::execvp(&cargs[0], &cargs);

    match result {
        Err(Errno::EACCES) => {
            println!("sush: {}: Permission denied", &args[0]);
            process::exit(126)
        },
        Err(Errno::ENOENT) => {
            println!("{}: command not found", &args[0]);
            process::exit(127)
        },
        Err(err) => {
            println!("Failed to execute. {err:?}");
            process::exit(127)
        }
        _ => panic!("SUSH INTERNAL ERROR (never come here)")
    }
}
