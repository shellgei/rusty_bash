//SPDX-FileCopyrightText: 2022-2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::job::Job;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

enum Status{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug, Clone, Default)]
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
        if feeder.starts_with(";;") || feeder.starts_with(";&") {
            ans.job_ends.push("".to_string());
            return true;
        }
        let len = feeder.scanner_job_end();
        let end = &feeder.consume(len);
        ans.job_ends.push(end.clone());
        ans.text += &end;
        len != 0
    }

    fn check_nest(&self, feeder: &mut Feeder, permit_empty: bool) -> Status {
        let nest = feeder.nest.last().unwrap();

        if nest.0 == "" && feeder.len() == 0 {
            return Status::NormalEnd;
        }

        match ( nest.1.iter().find(|e| feeder.starts_with(e)), self.pipeline_num() ) {
            ( Some(end), 0 ) => {
                if permit_empty {
                    return Status::NormalEnd;
                }
                return Status::UnexpectedSymbol(end.to_string())
            },
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

    fn unalias(&mut self, core: &mut ShellCore) {
        for a in core.alias_memo.iter().rev() {
            self.text = self.text.replace(&a.1, &a.0);
        }

        core.alias_memo.clear();
    }

    fn pipeline_num(&self) -> usize {
        self.jobs.iter().map(|j| j.pipelines.len()).sum()
    }

    fn read_heredoc(&mut self, feeder: &mut Feeder, core: &mut ShellCore) -> Result<(), ParseError> {
        for job in self.jobs.iter_mut() {
            job.read_heredoc(feeder, core)?;
        }
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore,
                 permit_empty: bool) -> Result<Option<Script>, ParseError> {
        let mut ans = Self::default();
        loop {
            while Self::eat_job(feeder, core, &mut ans)? 
                  && Self::eat_job_end(feeder, &mut ans) {}

            match ans.check_nest(feeder, permit_empty){
                Status::NormalEnd => {
                    ans.unalias(core);
                    ans.read_heredoc(feeder, core)?;
                    return Ok(Some(ans))
                },
                Status::NeedMoreLine => {
                    ans.read_heredoc(feeder, core)?;
                    feeder.feed_additional_line(core)?
                },
                Status::UnexpectedSymbol(s) => { //unexpected symbol
                    let _ = core.db.set_param("LINENO", &feeder.lineno.to_string(), None);
                    core.db.exit_status = 2;
                    return Err(ParseError::UnexpectedSymbol(s));
                },
            }
        }
    }
}
