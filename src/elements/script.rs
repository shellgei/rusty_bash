//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::{Feeder, ShellCore};

enum EndStatus{
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

    fn new() -> Script {
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

    fn check_nest(feeder: &mut Feeder, ends: &Vec<&str>, jobnum: usize) -> EndStatus {
        if let Some(end) = ends.iter().find(|e| feeder.starts_with(e)) {
            if jobnum == 0 {
                return EndStatus::UnexpectedSymbol(end.to_string());
            }
            return EndStatus::NormalEnd;
        }

        let other_ends = vec![")", "}", "then", "else", "fi", "elif", "do", "done"];
        if let Some(end) = other_ends.iter().find(|e| feeder.starts_with(e)) {
            return EndStatus::UnexpectedSymbol(end.to_string());
        }

        if ends.len() == 0 {
            EndStatus::NormalEnd
        }else{
            EndStatus::NeedMoreLine
        }
    }

    fn check_end(feeder: &mut Feeder, core: &mut ShellCore, jobnum: usize) -> EndStatus {
        if let Some(begin) = core.nest.last() {
            match begin.as_ref() {
                "(" => Self::check_nest(feeder, &vec![")"], jobnum),
                _ => EndStatus::NormalEnd,
            }
        }else{
            Self::check_nest(feeder, &vec![], jobnum)
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        let mut ans = Self::new();

        while Self::eat_job(feeder, core, &mut ans) {
            while Self::eat_job_end(feeder, &mut ans) {} //TODO: prohibit echo a;; 
        }

        match Self::check_end(feeder, core, ans.jobs.len()){
            EndStatus::UnexpectedSymbol(s) => {
                eprintln!("Unexpected token: {}", s);
                core.vars.insert("?".to_string(), "2".to_string());
                feeder.remaining = String::new();
                return None;
            },
            EndStatus::NeedMoreLine => {
                eprintln!("need more line");
                feeder.remaining = String::new();
                return None;
            },
            EndStatus::NormalEnd => {
                return Some( ans )
            }
        }
    }
}
