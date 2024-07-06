//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::unistd::Pid;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

#[derive(Debug)]
pub struct JobEntry {
    pids: Vec<Pid>,
    proc_statuses: Vec<WaitStatus>,
    display_status: String,
    text: String,
    change: bool,
}

fn wait_nonblock(pid: &Pid, status: &mut WaitStatus) {
    let waitflags = WaitPidFlag::WNOHANG 
                  | WaitPidFlag::WUNTRACED
                  | WaitPidFlag::WCONTINUED;

    match waitpid(*pid, Some(waitflags)) {
        Ok(s) => {
            if s == WaitStatus::StillAlive && still(status) {
                return;
            }

            *status = s;
        },
        _  => panic!("SUSHI INTERNAL ERROR (wrong pid wait)"),
    }
}

fn still(status: &WaitStatus) -> bool {
    match &status {
        WaitStatus::StillAlive    => true,
        WaitStatus::Stopped(_, _) => true,
        WaitStatus::Continued(__) => true,
        _ => false,
    }
}

impl JobEntry {
    pub fn new(pids: Vec<Option<Pid>>, text: &str) -> JobEntry {
        let len = pids.len();
        JobEntry {
            pids: pids.into_iter().flatten().collect(),
            proc_statuses: vec![ WaitStatus::StillAlive; len ],
            display_status: "Running".to_string(), 
            text: text.to_string(),
            change: false,
        }
    }

    pub fn update_status(&mut self) {
        let before = self.proc_statuses[0];
        for (status, pid) in self.proc_statuses.iter_mut().zip(&self.pids) {
            if still(status) {
                wait_nonblock(pid, status);
            }
        }
        self.change |= before != self.proc_statuses[0];

        if self.change {
            self.change_display_status(self.proc_statuses[0]);
        }
    }

    pub fn print(&self, id: usize) {
        println!("[{}]  {}     {}", id+1, &self.display_status, &self.text);
    }

    fn change_display_status(&mut self, after: WaitStatus) {
        self.display_status = match after {
            WaitStatus::Exited(_, _)                     => "Done",
            WaitStatus::Stopped(_, _)                    => "Stopped",
            WaitStatus::Continued(_)                     => "Running",
            WaitStatus::Signaled(_, signal::SIGKILL, _)  => "Killed",
            WaitStatus::Signaled(_, signal::SIGTERM, _)  => "Terminated",
            _ => return,
        }.to_string();
    }
}

impl ShellCore {
    pub fn jobtable_check_status(&mut self) {
        for e in self.job_table.iter_mut() {
            e.update_status();
        }
    }

    pub fn jobtable_print_status_change(&mut self) {
        for (i, e) in self.job_table.iter_mut().enumerate() {
            if e.change {
                e.print(i);
                e.change = false;
            }
        }

        self.job_table.retain(|e| still(&e.proc_statuses[0]));
    }
}
