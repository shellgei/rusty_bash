//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::pipeline::Pipeline;
use crate::{Feeder, proc_ctrl, ShellCore};
use crate::core::jobtable::JobEntry;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::exit;
use nix::unistd;
use nix::unistd::{Pid, ForkResult};

#[derive(Debug, Default, Clone)]
pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub pipeline_ends: Vec<String>,
    pub text: String,
}

impl Job {
    pub fn exec(&mut self, core: &mut ShellCore, bg: bool) -> Result<(), ExecError> {
        let pgid = match core.is_subshell {
            true  => unistd::getpgrp(),
            false => Pid::from_raw(0),
        };

        match bg {
            true  => self.exec_bg(core, pgid),
            false => self.exec_fg(core, pgid),
        }
    }

    fn exec_fg(&mut self, core: &mut ShellCore, pgid: Pid) -> Result<(), ExecError> {
        let mut do_next = true;
        for (pipeline, end) in self.pipelines.iter_mut()
                          .zip(self.pipeline_ends.iter()) {
            if core.return_flag {
                break;
            }

            if do_next {
                core.jobtable_check_status();
                let (pids, err) = pipeline.exec(core, pgid);
                //core.wait_pipeline(pids);
                proc_ctrl::wait_pipeline(core, pids);

                if let Some(e) = err {
                    return Err(e);
                }
            }
            do_next = (core.db.get_param("?").unwrap() == "0") == (end == "&&");
        }
        Ok(())
    }

    fn exec_bg(&mut self, core: &mut ShellCore, pgid: Pid) -> Result<(), ExecError> {
        let backup = core.tty_fd.as_ref().map(|fd| fd.try_clone().unwrap());
        core.tty_fd = None;

        let pids = if self.pipelines.len() == 1 {
            if self.pipelines[0].commands.len() == 1 {
                self.pipelines[0].commands[0].set_force_fork();
            }
            self.pipelines[0].exec(core, pgid).0
        }else{
            vec![self.exec_fork_bg(core, pgid)]
        };
        eprintln!("{}", &pids[0].unwrap().as_raw());
        core.job_table.push(JobEntry::new(pids, &self.text));

        core.tty_fd = backup;
        Ok(())
    }

    fn exec_fork_bg(&mut self, core: &mut ShellCore, pgid: Pid) -> Option<Pid> {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                core.initialize_as_subshell(Pid::from_raw(0), pgid);
                if let Err(e) = self.exec(core, false) {
                    e.print(core);
                }
                exit::normal(core)
            },
            Ok(ForkResult::Parent { child } ) => {
                proc_ctrl::set_pgid(core, child, pgid);
                Some(child) 
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
    }

    fn eat_blank_line(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        let com_num = feeder.scanner_comment();
        ans.text += &feeder.consume(com_num);
        if feeder.starts_with("\n") {
            ans.text += &feeder.consume(1);
            true
        }else{
            false
        }
    }

    fn eat_pipeline(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        match Pipeline::parse(feeder, core)? {
            Some(pipeline) => {
                ans.text += &pipeline.text.clone();
                ans.pipelines.push(pipeline);
                Ok(true)
            },
            None => Ok(false),
        }
    }

    fn eat_and_or(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_and_or(core);
        let end = feeder.consume(num);
        ans.pipeline_ends.push(end.clone());
        ans.text += &end;
        num != 0 //記号なしの場合にfalseが返る
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Job>, ParseError> {
        let mut ans = Self::default();
        while Self::eat_blank_line(feeder, &mut ans, core) {} 
        if ! Self::eat_pipeline(feeder, &mut ans, core)? {
            return Ok(None);
        }

        while Self::eat_and_or(feeder, &mut ans, core) { 
            loop {
                while Self::eat_blank_line(feeder, &mut ans, core) {} 
                if Self::eat_pipeline(feeder, &mut ans, core)? {
                    break;  
                }
                if feeder.len() == 0 {
                    feeder.feed_additional_line(core)?;
                }
            }
        }

        let com_num = feeder.scanner_comment();
        ans.text += &feeder.consume(com_num);
    
        match ans.pipelines.is_empty() {
            false => Ok(Some(ans)),
            true  => Ok(None),
        }
    }
}
