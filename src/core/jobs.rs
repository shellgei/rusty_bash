//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use super::job::Job;
use crate::elements::command::Command;
use super::proc;
//use nix::unistd;

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
        job.signaled_bg = true;
        job.id = self.backgrounds.len()+1;
        job.async_pids.push(pid);
        println!("{}", &job.status_string(job.id, 0));
        self.add_job(job.clone());
    }

    pub fn set_fg_job(&mut self, text: &String, commands: &Vec<Box<dyn Command>>) {
        self.foreground = Job::new(text, commands, false);
    }

    pub fn add_bg_job(&mut self, text: &String, commands: &Vec<Box<dyn Command>>) {
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

        self.foreground.status = 'D';
        pipestatus
    }

    pub fn wait_bg_job_at_foreground(&mut self, job_no: usize) -> Vec<i32> {
        if job_no == 0 {
            panic!("bash internal error (call the foreground job as a background)");
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

        self.backgrounds[pos].status = 'D';
        pipestatus
    }

    pub fn wait_process(&mut self, child: Pid) -> i32 {
        let (exit_status, status) = proc::wait_process(child);

        if status == 'S' {
            self.to_background(child);
        }

        exit_status
    } 

    pub fn add_job(&mut self, added: Job) {
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
