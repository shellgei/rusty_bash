//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;

use nix::sys::wait;
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use std::collections::HashMap;

pub struct ShellCore {
    pub history: Vec<String>,
    pub vars: HashMap<String, String>, 
    pub builtins: HashMap<String, fn(&mut ShellCore, args: &mut Vec<String>) -> i32>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore{
            history: Vec::new(),
            vars: HashMap::new(),
            builtins: HashMap::new(),
        };

        core.vars.insert("?".to_string(), "0".to_string());

        //core.builtins.insert("cd".to_string(), builtins::cd);
        core.builtins.insert("exit".to_string(), builtins::exit);

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

    pub fn run_builtin(&mut self, args: &mut Vec<String>) -> bool {
        if self.builtins.contains_key(&args[0]) {
            let func = self.builtins[&args[0]];
            let status = func(self, args);
            self.vars.insert("?".to_string(), status.to_string());
            true
        }else{
            false
        }
    }
}
