//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use crate::elements::command::Command;
use crate::ShellCore;

//[1]+  Running                 sleep 5 &
#[derive(Clone,Debug)]
pub struct Job {
    pub pids: Vec<Pid>,
    pub async_pids: Vec<Pid>,
    text: String,
    pub status: String,
    pub is_bg: bool,
    pub id: usize,
    pub mark: char, // '+': current, '-': previous, ' ': others
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
            async_pids: vec![],
            text: text.clone(),
            status: "Running".to_string(),
            is_bg: is_bg,
            id: 0,
            mark: '+',
        }
    }

    pub fn check_of_finish(&mut self) {
        let mut remain = vec![];

        while self.async_pids.len() > 0 {
            let p = self.async_pids.pop().unwrap();

            if ! ShellCore::check_process(p){
                remain.push(p);
            }
        }

        if remain.len() == 0 {
            self.status = "Done".to_string();
            print!("{}", self.status_string().clone());
        }

        self.async_pids = remain;
    }

    pub fn status_string(&self) -> String {
        format!("[{}]{} {} {}", &self.id, &self.mark, &self.status, &self.text)
    }

}
