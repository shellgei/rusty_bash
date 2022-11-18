//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use nix::sys::wait::{waitpid, WaitStatus};

pub struct ShellCore {
    pub history: Vec<String>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let conf = ShellCore{
            history: Vec::new(),
        };

        conf
    }

    pub fn wait_process(&mut self, child: Pid) {
        let exit_status = match waitpid(child, None) {//第2引数はオプション
            Ok(WaitStatus::Exited(_pid, status)) => {
                status
            },
            Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                128+signal as i32 
            },
            Ok(unsupported) => {
                eprintln!("Error: {:?}", unsupported);
                1
            },
            Err(err) => {
                panic!("Error: {:?}", err);
            },
        };

        eprintln!("終了ステータス: {}", exit_status);
    } 
}
