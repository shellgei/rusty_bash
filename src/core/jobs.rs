//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod job;

use nix::unistd::Pid;
use job::Job;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use crate::elements::command::Command;

//[1]+  Running                 sleep 5 &
#[derive(Clone,Debug)]
pub struct Jobs {
    pub foreground: Job,
    pub backgrounds: Vec<Job>, //0: current job, 1~: background jobs
}

impl Jobs {
    pub fn new() -> Jobs {
        Jobs {
            foreground: Job::new(&"".to_string(), &vec![], false),
            backgrounds: vec![],
        }
    }

    pub fn get_worst_priority(& self) -> u32 {
        let mut max = 0;

        for j in &self.backgrounds {
            if max <= j.priority {
                max = j.priority;
            }
        }
        max
    }

    pub fn get_top_priority_id(& self) -> (usize, usize) {
        let mut min = std::u32::MAX; 
        let mut id = 0;

        for j in &self.backgrounds {
            if min > j.priority {
                min = j.priority;
                id = j.id;
            }
        }

        min = std::u32::MAX; 
        let mut id_second = 0;

        for j in &self.backgrounds {
            if min > j.priority && id != j.id {
                min = j.priority;
                id_second = j.id;
            }
        }

        (id, id_second)
    }

    fn to_background(&mut self, pid: Pid){
        for j in self.backgrounds.iter_mut() {
            j.priority += 1;
        }

        let mut job = self.foreground.clone();
        job.status = 'S';
        job.id = self.backgrounds.len()+1;
        //job.mark = '+';
        job.async_pids.push(pid);
        println!("{}", &job.status_string(job.id, 0));
        self.add_job(job.clone());
    }

    pub fn set_fg_job(&mut self, text: &String, commands: &Vec<Box<dyn Command>>) {
        self.foreground = Job::new(text, commands, false);
    }

    pub fn add_bg_job(&mut self, text: &String, commands: &Vec<Box<dyn Command>>) {
        /*
        for j in self.backgrounds.iter_mut() {
            j.priority += 1;
        }*/

        let mut bgjob = Job::new(text, commands, true);
        bgjob.id = self.backgrounds.len() + 1;
        bgjob.priority = self.get_worst_priority() + 1;

        if let Some(pid) = commands.last().unwrap().get_pid() {
            eprintln!("[{}] {}", bgjob.id, pid);
            bgjob.async_pids.push(pid);
        }else{
            panic!("Bash internal error (before running background process)");
        }

        self.add_job(bgjob);
        return;
    }

    pub fn wait_fg_job(&mut self) -> Vec<i32> {
        let mut pipestatus = vec![];
        for p in self.foreground.pids.clone() {
            let exit_status = self.wait_process(p);
            pipestatus.push(exit_status);
        }

        /*
        if self.foreground.mark == '+' {
            for j in self.backgrounds.iter_mut() {
                if j.mark == '-' {
                    j.mark = '+';
                }
            }
        }*/

        self.foreground.status = 'D';
        pipestatus
    }

    pub fn wait_job(&mut self, job_no: usize) -> Vec<i32> {
        if job_no == 0 {
            return self.wait_fg_job();
        }

        let pos = job_no - 1;

        if self.backgrounds[pos].status != 'F' {
            return vec![];
        }

        let mut pipestatus = vec![];
        for p in self.backgrounds[pos].pids.clone() {
            let exit_status = self.wait_process(p);
            pipestatus.push(exit_status);
        }

        /*
        if self.backgrounds[pos].mark == '+' {
            for j in self.backgrounds.iter_mut() {
                if j.mark == '-' {
                    j.mark = '+';
                }
            }
        }*/

        self.backgrounds[pos].status = 'D';
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
        /*
        if added.mark == '+' {
            for job in self.backgrounds.iter_mut() {
                job.mark = if job.mark == '+' {'-'}else{' '};
            }
        }*/

        self.backgrounds.push(added);
    }

    pub fn remove_finished_jobs(&mut self) {
        while self.backgrounds.len() > 0 {
            let job = self.backgrounds.pop().unwrap();

            if job.status != 'I' && job.status != 'F' {
                self.backgrounds.push(job);
                break;
            }
        }
    }
}
