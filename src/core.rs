//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::sys::wait;
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use std::collections::HashMap; //追加

pub struct ShellCore {
    pub history: Vec<String>,
    pub vars: HashMap<String, String>, //追加
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore{ // mutに変更
            history: Vec::new(),
            vars: HashMap::new(), //追加
        };

        core.vars.insert("?".to_string(), "0".to_string()); //追加
        core
    }

    pub fn wait_process(&mut self, child: Pid) {
        let exit_status = match wait::waitpid(child, None) {
            Ok(WaitStatus::Exited(_pid, status)) => {
                status
            },
            Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                128+signal as i32
            },
            Ok(unsupported) => {
                eprintln!("Unsupported: {:?}", unsupported);
                1
            },
            Err(err) => {
                panic!("Error: {:?}", err);
            },
        };

        self.vars.insert("?".to_string(), exit_status.to_string()); //追加
    } 
}
