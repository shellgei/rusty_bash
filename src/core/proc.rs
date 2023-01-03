//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use std::fs;

use nix::sys::signal;
use nix::sys::signal::{Signal, SigHandler};

pub fn wait_process(child: Pid) -> i32 {
    let exit_status = match waitpid(child, Some(WaitPidFlag::WUNTRACED)) {
        Ok(WaitStatus::Exited(_pid, status)) => {
            status
        },
        Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
            eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
            128+signal as i32
        },
        Ok(WaitStatus::Stopped(_pid, signal)) => {
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

    exit_status
} 

pub fn check_async_process(pid: Pid) -> bool {
    match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::StillAlive) => false,
        Ok(_)                      => true, 
        _                          => {eprintln!("ERROR");true},
    }
}

pub fn check_status_from_file(pid: Pid) -> Option<char> {
    let path = format!("/proc/{}/stat", pid);
    match fs::read_to_string(path) {
        Ok(source) => {
            if let Some(s) = source.split(" ").nth(2) {
                Some(s.chars().nth(0).unwrap())
            }else{
                None
            }
        },
        _ => None,
    }
}

pub fn set_signals() {
    unsafe {
        signal::signal(Signal::SIGINT, SigHandler::SigDfl).unwrap();
        signal::signal(Signal::SIGTTIN, SigHandler::SigDfl).unwrap();
        signal::signal(Signal::SIGTTOU, SigHandler::SigDfl).unwrap();
        signal::signal(Signal::SIGTSTP, SigHandler::SigDfl).unwrap();
    }
}
