//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};

pub fn wait_process(child: Pid) -> (i32, char) {
    let exit_status = match waitpid(child, Some(WaitPidFlag::WUNTRACED)) {
        Ok(WaitStatus::Exited(_pid, status)) => {
            (status, 'D')
        },
        Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
            eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
            (128+signal as i32, 'D')
        },
        Ok(WaitStatus::Stopped(_pid, signal)) => {
            //self.to_background(pid);
            (128+signal as i32, 'S') 
        },
        Ok(unsupported) => {
            eprintln!("Error: {:?}", unsupported);
            (1, 'D')
        },
        Err(err) => {
            panic!("Error: {:?}", err);
        },
    };

    exit_status
} 

pub fn check_async_process(pid: Pid) -> bool {
    match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::StillAlive) => false,
        Ok(_)                      => true, 
        _                          => {eprintln!("ERROR");true},
    }
}
