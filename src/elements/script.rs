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

    fn check_nest(feeder: &mut Feeder, ends: &Vec<&str>, other_ends: &Vec<&str>, empty: bool) -> EndStatus {
        if let Some(end) = ends.iter().find(|e| feeder.starts_with(e)) {
            if end == &";;" || end == &";&" || end == &";;&" {
                return EndStatus::NormalEnd;
            }
            if empty {
                return EndStatus::UnexpectedSymbol(end.to_string());
            }
            return EndStatus::NormalEnd;
        }

        if let Some(end) = other_ends.iter().find(|e| feeder.starts_with(e)) {
            return EndStatus::UnexpectedSymbol(end.to_string());
        }
        return EndStatus::NeedMoreLine;
    }

    fn check_end(feeder: &mut Feeder, core: &mut ShellCore, empty: bool) -> EndStatus {
        let ends = vec![")", "}", "then", "else", "fi", "elif", "do", "done"];

        if let Some(begin) = core.nest.pop() {
            core.nest.push(begin.clone());
            return match begin.as_ref() {
                "(" => Self::check_nest(feeder, &vec![")"], &ends, empty),
                "{" => Self::check_nest(feeder, &vec!["}"], &ends, empty),
                "if" | "elif" => Self::check_nest(feeder, &vec!["then"], &ends, empty),
                "then" => Self::check_nest(feeder, &vec!["else", "fi", "elif"], &ends, empty),
                "else" => Self::check_nest(feeder, &vec!["fi"], &ends, empty),
                "while" => Self::check_nest(feeder, &vec!["do"], &ends, empty),
                "do" => Self::check_nest(feeder, &vec!["done"], &ends, empty),
                "_)" => Self::check_nest(feeder, &vec![";;", ";&", ";;&"], &ends, empty), // pattern in case
                _ => EndStatus::NormalEnd,
            };
        }

        if let Some(token) = ends.iter().find(|e| feeder.starts_with(e)) {
            return EndStatus::UnexpectedSymbol(token.to_string());
        }

        return EndStatus::NormalEnd;
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        let mut ans = Self::new();

        while Self::eat_job(feeder, core, &mut ans) {
            if ! Self::eat_job_end(feeder, &mut ans) {
                break;
            }
        }

        if feeder.remaining.len() == 0 {
            //eprintln!("{:?}", &ans);
            Some(ans)
        }else{
            eprintln!("ERROR");
            None
        }
    }
}
