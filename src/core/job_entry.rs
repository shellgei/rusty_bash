//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

/*
use nix::unistd::Pid;
use crate::elements::command::Command;
use super::proc;
*/

#[derive(Clone,Debug)]
pub struct JobEntry {
    pub text: String,
    /*
    pub signaled_bg: bool,
    pub pids: Vec<Pid>,
    pub async_pids: Vec<Pid>, //maybe not required.
    pub status: char, // S: stopped, R: running, D: done, I: invalid, F: fg
    pub id: usize,
    pub priority: u32,
    */
}

impl JobEntry {
    pub fn new() -> JobEntry {
        JobEntry {
            text: String::new(), 
        }
    }
}
