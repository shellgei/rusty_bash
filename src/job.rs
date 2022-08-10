//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use nix::sys::wait::waitpid;
use nix::sys::wait::WaitStatus;
use crate::abst_elems::PipelineElem;


//[1]+  Running                 sleep 5 &
#[derive(Clone)]
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

    pub fn wait(&mut self) -> Vec<i32> {
        if self.status == "Done" {
            return vec!();
        }

        let mut pipestatus = vec!();
        for p in &self.pids {
            pipestatus.push(Job::wait_pid(*p));
        }
        self.status = "Done".to_string();
        pipestatus
    }

    fn wait_pid(child: Pid) -> i32 {
        match waitpid(child, None).expect("Faild to wait child process.") {
            WaitStatus::Exited(_pid, status) => {
                status
            }
            WaitStatus::Signaled(pid, signal, _) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                128+signal as i32
            }
            _ => {
                panic!("Unknown error");
            }
        }
    }

    pub fn status_string(self) -> String {
        format!("{} {}", &self.status, &self.text)
    }

}
