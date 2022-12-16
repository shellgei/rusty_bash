//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use crate::elements::abst_command::AbstCommand;

//[1]+  Running                 sleep 5 &
#[derive(Clone)]
pub struct Job {
    pub pids: Vec<Pid>,
    text: String,
    pub status: String,
}

impl Job {
    pub fn new(text: &String, commands: &Vec<Box<dyn AbstCommand>>) -> Job {
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
        }
    }

    pub fn status_string(self) -> String {
        format!("{} {}", &self.status, &self.text)
    }

}
