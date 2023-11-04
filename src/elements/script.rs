//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::{Feeder, ShellCore};
use nix::unistd;
use nix::unistd::{ForkResult, Pid};
use super::{io, Pipe};
use super::io::redirect::Redirect;

enum Status{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug)]
pub struct Script {
    pub jobs: Vec<Job>,
    pub job_ends: Vec<String>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore, redirects: &mut Vec<Redirect>) {
        if redirects.iter_mut().all(|r| r.connect(true)){
            for (job, end) in self.jobs.iter_mut().zip(self.job_ends.iter()) {
                job.exec(core, end == "&");
            }
        }else{
            core.vars.insert("?".to_string(), "1".to_string());
        }
        redirects.iter_mut().rev().for_each(|r| r.restore());
    }

    pub fn fork_exec(&mut self, core: &mut ShellCore,pipe: &mut Pipe,
                     redirects: &mut Vec<Redirect>) -> Option<Pid> {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                core.is_subshell = true;
                core.set_pgid(Pid::from_raw(0), pipe.pgid);
                core.set_subshell_vars();
                io::connect(pipe, redirects);
                self.exec(core, &mut vec![]);
                core.exit()
            },
            Ok(ForkResult::Parent { child } ) => {
                core.set_pgid(child, pipe.pgid);
                pipe.parent_close();
                Some(child) //   core.wait_process(child);
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
    }

    pub fn new() -> Script {
        Script {
            text: String::new(),
            jobs: vec![],
            job_ends: vec![],
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
        let end = &feeder.consume(len);
        ans.job_ends.push(end.clone());
        ans.text += &end;
        len != 0
    }

    fn check_nest_end(feeder: &mut Feeder, ok_ends: &Vec<&str>, jobnum: usize) -> Status {
        if let Some(end) = ok_ends.iter().find(|e| feeder.starts_with(e)) {
            if jobnum == 0 {
                return Status::UnexpectedSymbol(end.to_string());
            }
            return Status::NormalEnd;
        }

        let ng_ends = vec![")", "}", "then", "else", "fi", "elif", "do", "done", "||", "&&", "|", "&"];
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
                    if ! feeder.feed_additional_line(core) {
                        break;
                    }
                },
            }
        }

        core.vars.insert("?".to_string(), "2".to_string());
        feeder.consume(feeder.len());
        return None;
    }
}
