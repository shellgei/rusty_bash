//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::{Feeder, ShellCore};

enum Status{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug, Clone)]
pub struct Script {
    pub jobs: Vec<Job>,
    pub job_ends: Vec<String>,
    text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for (job, end) in self.jobs.iter_mut().zip(self.job_ends.iter()) {
            job.exec(core, end == "&");
        }
    }

    pub fn get_text(&self) -> String { self.text.clone() }

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

    fn check_nest(feeder: &mut Feeder, jobnum: usize) -> Status {
        let nest = feeder.nest.last().expect("SUSHI INTERNAL ERROR (empty nest)");

        match ( nest.1.iter().find(|e| feeder.starts_with(e)), jobnum ) {
            ( Some(end), 0 ) => return Status::UnexpectedSymbol(end.to_string()),
            ( Some(_), _)    => return Status::NormalEnd,
            ( None, _)       => {}, 
        }

        let ng_ends = vec!["(", ")", "}", "then", "else", "if", "fi", "elif", "do", "done", "while", "||", "&&", "|", "&"];
        match ( ng_ends.iter().find(|e| feeder.starts_with(e)), nest.1.len() ) {
            (Some(end), _) => return Status::UnexpectedSymbol(end.to_string()),
            (None, 0)      => return Status::NormalEnd,
            (None, _)      => return Status::NeedMoreLine,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        let mut ans = Self::new();

        loop {
            while Self::eat_job(feeder, core, &mut ans) 
               && Self::eat_job_end(feeder, &mut ans) {}
    
            match Self::check_nest(feeder, ans.jobs.len()){
                Status::NormalEnd => {
                    return Some(ans)
                },
                Status::UnexpectedSymbol(s) => {
                    eprintln!("Unexpected token: {}", s);
                    core.set_param("?", "2");
                    break;
                },
                Status::NeedMoreLine => {
                    if ! feeder.feed_additional_line(core) {
                        break;
                    }
                },
            }
        }

        feeder.consume(feeder.len());
        return None;
    }
}
