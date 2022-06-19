//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore};
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use nix::sys::wait::waitpid;
use nix::sys::wait::WaitStatus;

pub trait ScriptElem {
    fn exec(&mut self, _conf: &mut ShellCore) { }
    fn set_pipe(&mut self, _pin: RawFd, _pout: RawFd, _pprev: RawFd) { }
    fn get_pid(&self) -> Option<Pid> { None }
    fn set_parent_io(&mut self) { }
    fn get_pipe_end(&mut self) -> RawFd { -1 }
    fn get_eoc_string(&mut self) -> String { "".to_string() }

    fn wait(&self, child: Pid, conf: &mut ShellCore) {
        match waitpid(child, None).expect("Faild to wait child process.") {
            WaitStatus::Exited(pid, status) => {
                conf.vars.insert("?".to_string(), status.to_string());
                if status != 0 { 
                    eprintln!("Pid: {:?}, Exit with {:?}", pid, status);
                }
            }
            WaitStatus::Signaled(pid, signal, _) => {
                conf.vars.insert("?".to_string(), (128+signal as i32).to_string());
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal)
            }
            _ => {
                eprintln!("Unknown error")
            }
        };
    }
}

