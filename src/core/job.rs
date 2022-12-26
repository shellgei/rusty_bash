//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use crate::elements::command::Command;

//[1]+  Running                 sleep 5 &
#[derive(Clone,Debug)]
pub struct Job {
    pub pids: Vec<Pid>,
    text: String,
    pub status: String,
    pub is_bg: bool,
    pub id: usize,
}

impl Job {
    pub fn new(text: &String, commands: &Vec<Box<dyn Command>>, is_bg: bool) -> Job {
        let mut pids = vec![];
        for c in commands {
            if let Some(p) = c.get_pid() {
                pids.push(p);
            }
        }

        Job {
            pids: pids,
            text: text.clone(),
            status: "Running".to_string(),
            is_bg: is_bg,
            id: 0,
        }
    }

    pub fn status_string(&self) -> String {
        format!("[{}] {} {}", &self.id, &self.status, &self.text)
    }

}
