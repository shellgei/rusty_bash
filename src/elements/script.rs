//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;

enum Status{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug, Default)]
pub struct Script {
    pub jobs: Vec<Job>,
    pub job_ends: Vec<String>,
    text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        for (job, end) in self.jobs.iter_mut().zip(self.job_ends.iter()) {
            job.exec(core, end == "&")?;
        }
        Ok(())
    }

    pub fn get_text(&self) -> String { self.text.clone() }

    fn eat_job(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Script) -> Result<bool, ParseError> {
        if let Some(job) = Job::parse(feeder, core)? {
            ans.text += &job.text.clone();
            ans.jobs.push(job);
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_job_end(feeder: &mut Feeder, ans: &mut Script) -> bool {
        let len = feeder.scanner_job_end();
        let end = &feeder.consume(len);
        ans.job_ends.push(end.clone());
        ans.text += end;
        len != 0
    }

    fn check_nest(&self, feeder: &mut Feeder) -> Status {
        let nest = feeder.nest.last().expect("SUSHI INTERNAL ERROR (empty nest)");

        if nest.0.is_empty() && feeder.len() == 0 {
            return Status::NormalEnd;
        }

        match ( nest.1.iter().find(|e| feeder.starts_with(e)), self.jobs.len() ) {
            ( Some(end), 0 ) => return Status::UnexpectedSymbol(end.to_string()),
            ( Some(_), _)    => return Status::NormalEnd,
            ( None, _)       => {}, 
        }

        if feeder.len() > 0 {
            let remaining = feeder.consume(feeder.len());
            let first_token = remaining.split(" ").nth(0).unwrap().to_string();
            return Status::UnexpectedSymbol(first_token);
        }

        Status::NeedMoreLine
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();

        loop {
            while Self::eat_job(feeder, core, &mut ans)? 
               && Self::eat_job_end(feeder, &mut ans) {}
    
            match ans.check_nest(feeder){
                Status::NormalEnd => return Ok(Some(ans)),
                Status::NeedMoreLine => feeder.feed_additional_line(core)?,
                Status::UnexpectedSymbol(s) => {
                    core.db.set_param("?", "2").unwrap();
                    return Err(ParseError::UnexpectedSymbol(s.clone()));
                },
            }
        }
    }
}
