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
    pub text: String,
    pub status: char, // S: stopped, R: running, D: done, F: fg
    pub is_bg: bool,
    pub is_waited: bool,
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
            status: 'R',
            is_bg: is_bg,
            is_waited: false,
            id: 0,
            mark: ' ',
        }
    }

    pub fn check_of_finish(&mut self) {
        if self.is_waited || self.status == 'F' {
            return; 
        }

        let mut remain = vec![];

        while self.async_pids.len() > 0 {
            let p = self.async_pids.pop().unwrap();

            if ! ShellCore::check_async_process(p){
                remain.push(p);
            }
        }

        if remain.len() == 0 {
            self.status = 'D';
        }

        self.async_pids = remain;
    }

    pub fn status_string(&self) -> String {
        let status = match self.status {
            'D' => "Done",
            'S' => "Stopped",
            'R' => "Running",
            _   => "ERROR",
        };
        format!("[{}]{} {} {}", &self.id, &self.mark, status, &self.text.trim_end())
    }

    pub fn print_status(&mut self) {
        if self.status == 'P' || self.status == 'F' {
            return;
        }

        println!("{}", &self.status_string());
        if self.status == 'D' {
            self.status = 'P';
        }
    }
}
