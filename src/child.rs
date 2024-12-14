//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{exit, ShellCore};
use crate::utils::error;
use nix::sys::{resource, signal, wait};
use nix::sys::wait::{WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::sync::atomic::Ordering::Relaxed;

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
        let ws = core.wait_process(pid.expect("SUSHI INTERNAL ERROR (no pid)"));
        ans.push(ws);

        pipestatus.push(core.data.get_param("?"));
    }

    if time {
        core.show_time();
    }
    core.set_foreground();
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

pub fn wait_process(core: &mut ShellCore, child: Pid) -> WaitStatus {
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
