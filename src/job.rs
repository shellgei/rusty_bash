//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use nix::sys::wait::waitpid;
use crate::ShellCore;
use nix::sys::wait::WaitStatus;
use crate::abst_elems::PipelineElem;


#[derive(Clone)]
//[1]+  Running                 sleep 5 &
pub struct Job {
    pids: Vec<Pid>,
    text: String,
    status: String,
}

impl Job {
    pub fn new(text: &String, commands: &Vec<Box<dyn PipelineElem>>) -> Job {
        let mut pids = vec!();
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

    pub fn set_pid(&mut self, commands: &Vec<Box<dyn PipelineElem>>) {
        for c in commands {
            if let Some(p) = c.get_pid() {
                self.pids.push(p);
            }
        }
    }

    pub fn wait(self, conf: &mut ShellCore) {
        for p in self.pids {
            wait(p, conf);
        }
    }
}

pub fn wait(child: Pid, conf: &mut ShellCore) {
    match waitpid(child, None).expect("Faild to wait child process.") {
        WaitStatus::Exited(_pid, status) => {
            conf.vars.insert("?".to_string(), status.to_string());
        }
        WaitStatus::Signaled(pid, signal, _) => {
            conf.vars.insert("?".to_string(), (128+signal as i32).to_string());
            eprintln!("Pid: {:?}, Signal: {:?}", pid, signal)
        }
        _ => {
            eprintln!("Unknown error")
        }
    };
}
