//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::{Feeder, ShellCore};
use nix::unistd;
use nix::unistd::ForkResult;
use super::Pipe;
use super::io::redirect::Redirect;
use std::process;

enum Status{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug)]
pub struct Script {
    pub jobs: Vec<Job>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for job in self.jobs.iter_mut() {
            job.exec(core);
        }
    }

    pub fn fork_exec(&mut self, core: &mut ShellCore,pipe: &mut Pipe,
                                redirects: &mut Vec<Redirect>) {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                let pid = nix::unistd::getpid();
                core.vars.insert("BASHPID".to_string(), pid.to_string());
<<<<<<< HEAD
                if ! redirects.iter_mut().all(|r| r.connect(false)){
                    core.exit();
=======
                if ! redirects.iter_mut().all(|r| r.connect()){
                    process::exit(1);
>>>>>>> cec02b5f2a1464c42bbd5ac2c1077c0dcdee185b
                }
                pipe.connect();
                self.exec(core);
                core.exit();
            },
            Ok(ForkResult::Parent { child } ) => {
                pipe.parent_close();
                core.wait_process(child);
            },
            Err(err) => panic!("Failed to fork. {}", err),
        }
    }

    pub fn new() -> Script {
        Script {
            text: String::new(),
            jobs: vec![]
        }
    }

    fn eat_job(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Script) -> bool {
        if let Some(job) = Job::parse(feeder, core){
            ans.text += &job.text.clone();
            ans.jobs.push(job);
            true
        }else{
            false
        }
    }

    fn eat_job_end(feeder: &mut Feeder, ans: &mut Script) -> bool {
        let len = feeder.scanner_job_end();
        if len > 0 {
            ans.text += &feeder.consume(len);
            true
        }else{
            false
        }
    }

    fn check_nest_end(feeder: &mut Feeder, ok_ends: &Vec<&str>, jobnum: usize) -> Status {
        if let Some(end) = ok_ends.iter().find(|e| feeder.starts_with(e)) {
            if jobnum == 0 {
                return Status::UnexpectedSymbol(end.to_string());
            }
            return Status::NormalEnd;
        }

        let ng_ends = vec![")", "}", "then", "else", "fi", "elif", "do", "done"];
        if let Some(end) = ng_ends.iter().find(|e| feeder.starts_with(e)) {
            return Status::UnexpectedSymbol(end.to_string());
        }

        if ok_ends.len() == 0 {
            Status::NormalEnd
        }else{
            Status::NeedMoreLine
        }
    }

    fn check_nest(feeder: &mut Feeder, core: &mut ShellCore, jobnum: usize) -> Status {
        if let Some(begin) = core.nest.last() {
            match begin.as_ref() {
                "(" => Self::check_nest_end(feeder, &vec![")"], jobnum),
                "{" => Self::check_nest_end(feeder, &vec!["}"], jobnum),
                _ => Status::NormalEnd,
            }
        }else{
            Self::check_nest_end(feeder, &vec![], jobnum)
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        let mut ans = Self::new();

        loop {
            while Self::eat_job(feeder, core, &mut ans) 
               && Self::eat_job_end(feeder, &mut ans) {}
    
            match Self::check_nest(feeder, core, ans.jobs.len()){
                Status::NormalEnd => {
                    return Some(ans)
                },
                Status::UnexpectedSymbol(s) => {
                    eprintln!("Unexpected token: {}", s);
                    break;
                },
                Status::NeedMoreLine => {
                    if feeder.feed_additional_line(core) {
                        continue;
                    }
                    eprintln!("bash: syntax error: unexpected end of file");
                    break;
                },
            }
        }

        core.vars.insert("?".to_string(), "2".to_string());
        feeder.consume(feeder.len());
        return None;
    }
}
