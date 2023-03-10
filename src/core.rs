//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;

use nix::sys::wait;
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use std::collections::HashMap;
use std::process;
use std::os::linux::fs::MetadataExt;
use std::path::Path;

pub struct ShellCore {
    pub history: Vec<String>,
    pub flags: String,
    pub vars: HashMap<String, String>, 
    pub builtins: HashMap<String, fn(&mut ShellCore, &mut Vec<String>) -> i32>,
}

fn is_interactive(pid: u32) -> bool {
    let std_path = format!("/proc/{}/fd/0", pid);
    match Path::new(&std_path).metadata() {
        Ok(metadata) => metadata.st_mode() == 8592,
        Err(err) => panic!("{}", err),
    }
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore{
            history: Vec::new(),
            flags: String::new(),
            vars: HashMap::new(),
            builtins: HashMap::new(),
        };

        let pid = process::id();
        if is_interactive(pid) {
            core.flags += "i";
        }
        core.vars.insert("$".to_string(), pid.to_string());

        core.vars.insert("?".to_string(), "0".to_string());

        core.builtins.insert("cd".to_string(), builtins::cd);
        core.builtins.insert("exit".to_string(), builtins::exit);

        core
    }

    pub fn has_flag(&self, flag: char) -> bool {
        if let Some(_) = self.flags.find(flag) {
            return true;
        }
        false
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
        if ! self.builtins.contains_key(&args[0]) {
            return false;
        }

        let func = self.builtins[&args[0]];
        let status = func(self, args);
        self.vars.insert("?".to_string(), status.to_string());
        true
    }

    pub fn exit(&self) -> i32 {
        let exit_status = match self.vars["?"].parse::<i32>() {
            Ok(n)  => n%256,
            Err(_) => {
                eprintln!("sush: exit: {}: numeric argument required", self.vars["?"]);
                2
            },
        };

        process::exit(exit_status)
    }
}
