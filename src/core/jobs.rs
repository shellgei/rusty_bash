//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod job;

use nix::unistd::Pid;
use crate::elements::command::Command;
use crate::ShellCore;
use job::Job;

//[1]+  Running                 sleep 5 &
#[derive(Clone,Debug)]
pub struct Jobs {
    pub backgrounds: Vec<Job>, //0: current job, 1~: background jobs
}

impl Jobs {
}
