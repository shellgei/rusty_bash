//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::unistd::Pid;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

#[derive(Debug)]
pub struct JobEntry {
    pub id: usize,
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

fn wait_block(pid: &Pid, status: &mut WaitStatus) {
    let waitflags = WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED;

    match waitpid(*pid, Some(waitflags)) {
        Ok(s) => {
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
    pub fn new(pids: Vec<Option<Pid>>, statuses: &Vec<WaitStatus>,
               text: &str, status: &str, id: usize) -> JobEntry {
        JobEntry {
            id: id,
            pids: pids.into_iter().flatten().collect(),
            proc_statuses: statuses.to_vec(),
            display_status: status.to_string(),
            text: text.to_string(),
            change: false,
        }
    }

    pub fn update_status(&mut self, wait: bool) {
        let before = self.proc_statuses[0];
        for (status, pid) in self.proc_statuses.iter_mut().zip(&self.pids) {
            if still(status) {
                match wait {
                    true  => wait_block(pid, status),
                    false => wait_nonblock(pid, status),
                }
            }
        }
        self.change |= before != self.proc_statuses[0];

        /* check stopped processes */
        let mut stopped = false;
        for s in &self.proc_statuses {
            if let WaitStatus::Stopped(_, _) = s {
                stopped = true;
                break;
            }
        }

        if stopped {
            self.display_status = "Stopped".to_string();
            return;
        }

        if ! stopped && self.display_status == "Stopped" || self.change {
            self.change_display_status(self.proc_statuses[0]);
        }

    }

    pub fn print(&self, priority: &Vec<usize>) {
        if priority[0] == self.id {
            println!("[{}]+  {}     {}", self.id, &self.display_status, &self.text);
        }else if priority.len() > 1 && priority[1] == self.id {
            println!("[{}]-  {}     {}", self.id, &self.display_status, &self.text);
        }else {
            println!("[{}]   {}     {}", self.id, &self.display_status, &self.text);
        }
    }

    fn display_status_on_signal(signal: &signal::Signal, coredump: bool) -> String {
        let coredump_msg = match coredump {
            true  => "    (core dumped)",
            false => "",
        };

        let msg = match signal {
            signal::SIGHUP    => "Hangup",
            signal::SIGINT    => "Interrupt",
            signal::SIGQUIT   => "Quit",
            signal::SIGILL    => "Illeagal instruction",
            signal::SIGTRAP   => "Trace/breakpoint trap",
            signal::SIGABRT   => "Aborted",
            signal::SIGBUS    => "Bus error",
            signal::SIGFPE    => "Floating point exception",
            signal::SIGKILL   => "Killed",
            signal::SIGUSR1   => "User defined signal 1",
            signal::SIGSEGV   => "Segmentation fault",
            signal::SIGUSR2   => "User defined signal 2",
            signal::SIGPIPE   => "Broken pipe",
            signal::SIGALRM   => "Alarm clock",
            signal::SIGTERM   => "Terminated",
          //  signal::SIGSTKFLT => "Stack fault",           not in macOS
            signal::SIGXCPU   => "CPU time limit exceeded",
            signal::SIGXFSZ   => "File size limit exceeded",
            signal::SIGVTALRM => "Virtual timer expired",
            signal::SIGPROF   => "Profiling timer expired",
          //  signal::SIGPWR    => "Power failure",         not in macOS
            signal::SIGSYS    => "Bad system call",
            _ => "",
        };

        (msg.to_owned() + coredump_msg).to_string()
    }

    fn change_display_status(&mut self, after: WaitStatus) {
        self.display_status = match after {
            WaitStatus::Exited(_, _)                  => "Done".to_string(),
            WaitStatus::Stopped(_, _)                 => "Stopped".to_string(),
            WaitStatus::Continued(_)                  => "Running".to_string(),
            WaitStatus::Signaled(_, signal, coredump) =>
                Self::display_status_on_signal(&signal, coredump),
            _ => return,
        }
    }

    pub fn send_cont(&mut self) {
        for pid in &self.pids {
            let _ = signal::kill(*pid, signal::SIGCONT);            
        }
    }
}

impl ShellCore {
    pub fn jobtable_check_status(&mut self) {
        for e in self.job_table.iter_mut() {
            e.update_status(false);
        }
    }

    pub fn jobtable_print_status_change(&mut self) {
        for e in self.job_table.iter_mut() {
            if e.change {
                e.print(&self.job_table_priority);
                e.change = false;
            }
        }

        self.job_table.retain(|e| still(&e.proc_statuses[0]) || e.display_status == "Stopped");

        let ids = self.job_table.iter().map(|j| j.id).collect::<Vec<usize>>();
        self.job_table_priority.retain(|id| ids.contains(id) );
    }

    pub fn generate_new_job_id(&self) -> usize {
        match self.job_table.last() {
            None      => 1,
            Some(job) => job.id + 1,
        }
    }
}
