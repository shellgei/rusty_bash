//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{close, dup2, read};
use std::os::unix::prelude::RawFd;
use std::fs::OpenOptions;
use std::os::unix::io::IntoRawFd;
use nix::unistd::Pid;
use nix::sys::wait::WaitPidFlag;
use nix::sys::wait::{waitpid, WaitStatus};
use crate::ShellCore;

pub struct FileDescs {
    pub pipein: RawFd,
    pub pipeout: RawFd,
    pub prevpipein: RawFd,
}

impl FileDescs {
    pub fn new() -> FileDescs {
        FileDescs {
            pipein: -1,
            pipeout: -1,
            prevpipein: -1,
        }
    }

    pub fn no_connection(&self) -> bool {
        false
    }

    pub fn set_child_io(&self) {
    }
}

pub fn dup_and_close(from: RawFd, to: RawFd){
    close(to).expect(&("Can't close fd: ".to_owned() + &to.to_string()));
    dup2(from, to).expect("Can't copy file descriptors");
    close(from).expect(&("Can't close fd: ".to_owned() + &from.to_string()));
}


pub fn set_parent_io(pout: RawFd) {
    if pout >= 0 {
        close(pout).expect("Cannot close outfd");
    };
}

pub fn read_pipe(pin: RawFd, pid: Pid, conf: &mut ShellCore) -> String {
    let mut ans = "".to_string();
    let mut ch = [0;1000];

    loop {
        while let Ok(n) = read(pin, &mut ch) {
            ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
            match waitpid(pid, Some(WaitPidFlag::WNOHANG)).expect("Faild to wait child process.") {
                WaitStatus::StillAlive => {
                    continue;
                },
                WaitStatus::Exited(_pid, status) => {
                    conf.set_var("?", &status.to_string());
                    break;
                },
                WaitStatus::Signaled(pid, signal, _) => {
                    conf.set_var("?", &(128+signal as i32).to_string());
                    eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                    break;
                },
                _ => {
                    break;
                },
            };
        }
        return ans;
    }
}
