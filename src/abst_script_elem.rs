//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore};
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use nix::sys::wait::waitpid;
use nix::sys::wait::WaitStatus;
use crate::utils_io::read_pipe;

pub trait ScriptElem {
    fn exec(&mut self, _conf: &mut ShellCore, _substitution: bool) { }
    fn set_pipe(&mut self, _pin: RawFd, _pout: RawFd, _pprev: RawFd) { }
    fn get_pid(&self) -> Option<Pid> { None }
    fn set_parent_io(&mut self) { }
    fn get_pipe_end(&mut self) -> RawFd { -1 }
    fn get_pipe_out(&mut self) -> RawFd { -1 }
    fn get_eoc_string(&mut self) -> String { "".to_string() }

    fn get_substitution_text(&mut self) -> String { "".to_string() }
}

pub fn wait(child: Pid, conf: &mut ShellCore, inpipe: RawFd) -> String {
    let mut ans = "".to_string();
    if inpipe != -1 {
        ans += &read_pipe(inpipe);
    }

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

    ans
}
