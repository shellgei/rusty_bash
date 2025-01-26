//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::exec::ExecError;
use nix::unistd;
use nix::unistd::Pid;
use nix::sys::signal;
use nix::sys::wait;
use nix::sys::wait::{WaitPidFlag, WaitStatus};

#[derive(Debug)]
pub struct JobEntry {
    pub id: usize,
    pids: Vec<Pid>,
    proc_statuses: Vec<WaitStatus>,
    pub display_status: String,
    pub text: String,
    change: bool,
}

fn wait_nonblock(pid: &Pid, status: &mut WaitStatus) -> Result<(), ExecError> {
    let waitflags = WaitPidFlag::WNOHANG 
                  | WaitPidFlag::WUNTRACED
                  | WaitPidFlag::WCONTINUED;

    let s = wait::waitpid(*pid, Some(waitflags))?;
    if s != WaitStatus::StillAlive || ! still(status) {
        *status = s;
    }
    Ok(())
}

fn wait_block(pid: &Pid, status: &mut WaitStatus) -> Result<i32, ExecError> {
    *status = wait::waitpid(*pid, Some(WaitPidFlag::WUNTRACED))?;
    let exit_status = match status {
        WaitStatus::Exited(_, es) => *es,
        WaitStatus::Stopped(_, _) => 148,
        WaitStatus::Signaled(_, sig, _) => *sig as i32 + 128,
        _ => 1,
    };

    Ok(exit_status)
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

    pub fn update_status(&mut self, wait: bool) -> Result<i32, ExecError> {
        let mut exit_status = 0;
        let before = self.proc_statuses[0];
        for (status, pid) in self.proc_statuses.iter_mut().zip(&self.pids) {
            if still(status) {
                match wait {
                    true  => exit_status = wait_block(pid, status)?,
                    false => {wait_nonblock(pid, status)?;},
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
            return Ok(148);
        }

        if ! stopped && self.display_status == "Stopped" || self.change {
            self.change_display_status(self.proc_statuses[0]);
        }

        Ok(exit_status)
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
            let _ = signal::kill(Pid::from_raw(-1 * i32::from(*pid)), signal::SIGCONT);            
        }
    }

    pub fn solve_pgid(&self) -> Pid {
        for pid in &self.pids {
            match unistd::getpgid(Some(*pid)) {
                Ok(pgid) => return pgid, 
                _ => {}, 
            }
        }
        Pid::from_raw(0)
    }
}

impl ShellCore {
    pub fn jobtable_check_status(&mut self) -> Result<(), ExecError> {
        for e in self.job_table.iter_mut() {
            e.update_status(false)?;
        }
        Ok(())
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

impl ShellCore {
    pub fn get_stopped_job_commands(&self) -> Vec<String> {
        self.job_table.iter().map(|j| j.text.split(' ').nth(0).unwrap().to_string()).collect()
    }
}
