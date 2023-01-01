//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod job;

use nix::unistd::Pid;
use job::Job;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};

//[1]+  Running                 sleep 5 &
#[derive(Clone,Debug)]
pub struct Jobs {
    pub backgrounds: Vec<Job>, //0: current job, 1~: background jobs
}

impl Jobs {
    fn to_background(&mut self, pid: Pid){
        let mut job = self.backgrounds[0].clone();
        job.status = 'S';
        job.id = self.backgrounds.len();
        job.mark = '+';
        job.async_pids.push(pid);
        println!("{}", &job.status_string());
        self.add_job(job.clone());
    }

    pub fn wait_job(&mut self, job_no: usize) -> Vec<i32> {
        if self.backgrounds[job_no].status != 'F' {
            return vec![];
        }

        let mut pipestatus = vec![];
        for p in self.backgrounds[job_no].pids.clone() {
            let exit_status = self.wait_process(p);
            pipestatus.push(exit_status);
        }

        //let plus = self.jobs[job_no].mark == '+';
        if self.backgrounds[job_no].mark == '+' {
        //if plus {
            for j in self.backgrounds.iter_mut() {
                if j.mark == '-' {
                    j.mark = '+';
                }
            }
        }

        //self.set_var("PIPESTATUS", &pipestatus.join(" "));
        self.backgrounds[job_no].status = 'D';
        pipestatus
    }

    pub fn wait_process(&mut self, child: Pid) -> i32 {
        let exit_status = match waitpid(child, Some(WaitPidFlag::WUNTRACED)) {
            Ok(WaitStatus::Exited(_pid, status)) => {
                status
            },
            Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                128+signal as i32 
            },
            Ok(WaitStatus::Stopped(pid, signal)) => {
                self.to_background(pid);
                128+signal as i32 
            },
            Ok(unsupported) => {
                eprintln!("Error: {:?}", unsupported);
                1
            },
            Err(err) => {
                panic!("Error: {:?}", err);
            },
        };

        exit_status
    } 

    pub fn add_job(&mut self, added: Job) {
        if added.mark == '+' {
            for job in self.backgrounds.iter_mut() {
                job.mark = if job.mark == '+' {'-'}else{' '};
            }
        }

        self.backgrounds.push(added);
    }
}
