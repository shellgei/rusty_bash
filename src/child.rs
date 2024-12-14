//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{exit, ShellCore};
use crate::utils::error;
use nix::unistd;
use nix::sys::{signal, wait};
use nix::sys::signal::{Signal, SigHandler};
use nix::sys::wait::{WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::sync::atomic::Ordering::Relaxed;

fn ignore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigIgn) }
        .expect("sush(fatal): cannot ignore signal");
}

fn restore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigDfl) }
        .expect("sush(fatal): cannot restore signal");
}

pub fn wait_pipeline(core: &mut ShellCore, pids: Vec<Option<Pid>>,
                     exclamation: bool, time: bool) -> Vec<WaitStatus> {
    if pids.len() == 1 && pids[0] == None {
        if time {
            core.show_time();
        }
        if exclamation {
            core.flip_exit_status();
        }
        core.check_e_option();
        return vec![];
    }

    let mut pipestatus = vec![];
    let mut ans = vec![];
    for pid in &pids {
        let ws = wait_process(core, pid.expect("SUSHI INTERNAL ERROR (no pid)"));
        ans.push(ws);

        pipestatus.push(core.data.get_param("?"));
    }

    if time {
        core.show_time();
    }
    set_foreground(core);
    core.data.set_layer_array("PIPESTATUS", &pipestatus, 0);

    if core.options.query("pipefail") {
        pipestatus.retain(|e| e != "0");

        if pipestatus.len() != 0 {
            core.data.set_param("?", &pipestatus.last().unwrap());
        }
    }

    if exclamation {
        core.flip_exit_status();
    }

    core.check_e_option();

    ans
}

fn wait_process(core: &mut ShellCore, child: Pid) -> WaitStatus {
    let waitflags = match core.is_subshell {
        true  => None,
        false => Some(WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED)
    };

    let ws = wait::waitpid(child, waitflags);

    let exit_status = match ws {
        Ok(WaitStatus::Exited(_pid, status)) => status,
        Ok(WaitStatus::Signaled(pid, signal, coredump)) => error::signaled(pid, signal, coredump),
        Ok(WaitStatus::Stopped(pid, signal)) => {
            eprintln!("Stopped Pid: {:?}, Signal: {:?}", pid, signal);
            148
        },
        Ok(unsupported) => {
            let msg = format!("Unsupported wait status: {:?}", unsupported);
            error::print(&msg, core);
            1
        },
        Err(err) => {
            let msg = format!("Error: {:?}", err);
            exit::internal(&msg);
        },
    };

    if exit_status == 130 {
        core.sigint.store(true, Relaxed);
    }
    core.data.set_layer_param("?", &exit_status.to_string(), 0); //追加
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

    ignore_signal(Signal::SIGTTOU); //SIGTTOUを無視
    unistd::tcsetpgrp(fd, pgid)
        .expect(&error::internal("cannot get the terminal"));
    restore_signal(Signal::SIGTTOU); //SIGTTOUを受け付け
}

pub fn set_pgid(core :&ShellCore, pid: Pid, pgid: Pid) {
    let _ = unistd::setpgid(pid, pgid);
    if pid.as_raw() == 0 && pgid.as_raw() == 0 { //以下3行追加
        set_foreground(core);
    }
}

