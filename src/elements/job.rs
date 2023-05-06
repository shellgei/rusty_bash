//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::pipeline::Pipeline;
use crate::utils::blue_string;
use crate::core::proc;
use nix::unistd::{ForkResult};
use nix::unistd;
use std::process::exit;
use super::command::simple::SimpleCommand;
use crate::operators::ControlOperator;

#[derive(Debug)]
pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub pipeline_ends: Vec<ControlOperator>,
    pub text: String,
    pub is_bg: bool,
}

impl Job {
    pub fn exec(&mut self, core: &mut ShellCore) {
        if self.is_bg {
            if self.pipeline_ends[0] == ControlOperator::And || self.pipeline_ends[0] == ControlOperator::Or {
               self.exec_and_or_bg_job(core);
               return;
            }else{ //single pipeline with &
                self.pipelines[0].is_bg = true;
                self.pipelines[0].text = self.text.clone(); //to show "&" at the end of the pipeline
            }
        }

        self.exec_job(core);
    }

    fn exec_job(&mut self, core: &mut ShellCore) {
        let mut eop = ControlOperator::NoChar;
        for (i, p) in self.pipelines.iter_mut().enumerate() {
            if core.has_flag('d') {
                eprintln!("{}", blue_string(&p.get_text()));
            }

            let status = core.get_var("?") == "0";
           
            if (status && eop == ControlOperator::Or) || (!status && eop == ControlOperator::And) {
                eop = self.pipeline_ends[i].clone();
                continue;
            }
            p.exec(core);
            eop = self.pipeline_ends[i].clone();
        }
    }

    fn exec_and_or_bg_job(&mut self, core: &mut ShellCore) {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                core.set_var("BASHPID", &nix::unistd::getpid().to_string());
                proc::set_signals();
                let pid = nix::unistd::getpid();
                let _ = unistd::setpgid(pid, pid);

                self.exec_job(core);


                exit(core.vars["?"].parse::<i32>().unwrap());
            },
            Ok(ForkResult::Parent { child } ) => {
                let mut com = SimpleCommand::new();
                com.group_leader = true;
                com.pid = Some(child);
                core.jobs.add_bg_job(&self.text, &vec!(Box::new(com)));
                return;
            },
            Err(err) => panic!("Failed to fork. {}", err),
        }
    }

    pub fn new() -> Job{
        Job {
            pipelines: vec![],
            pipeline_ends: vec![],
            text: "".to_string(),
            is_bg: false,
        }
    }

    fn check_job_end(feeder: &mut Feeder, ans: &mut Job) {
        let (n, op) = feeder.scanner_control_op();
        if let Some(p) = op {
            if &p == &ControlOperator::Semicolon || &p == &ControlOperator::BgAnd {
                ans.is_bg = &p == &ControlOperator::BgAnd;
                ans.text += &feeder.consume(n);
            }
        }
    }

    fn set_pipelineend(feeder: &mut Feeder, ans: &mut Job) -> bool {
        let (n, op) = feeder.scanner_control_op();
        if let Some(p) = op {
            if &p != &ControlOperator::And && &p != &ControlOperator::Or {
                ans.pipeline_ends.push(ControlOperator::NoChar);
                return true;
            }
            ans.pipeline_ends.push(p.clone());
            ans.text += &feeder.consume(n);
        }else{
            ans.pipeline_ends.push(ControlOperator::NoChar);
        }

        false
    }

    pub fn eat_pipeline(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Job) -> bool {
        let mut go_next = true;

        if let Some(result) = Pipeline::parse(feeder, core) {
            ans.text += &result.text;
            ans.pipelines.push(result);

            if Job::set_pipelineend(feeder, ans){
                go_next = false;
            }
        }
        else {
            go_next = false;
        }

        go_next
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Job> {
        if feeder.len() == 0 {
            return None;
        };
    
        let backup = feeder.clone();

        let mut ans = Job::new();
        ans.text += &feeder.consume_comment_multiline();
        while Job::eat_pipeline(feeder, core, &mut ans) {
            ans.text += &feeder.consume_comment_multiline();
            if feeder.len() == 0 {
                break;
            }
        }

        Self::check_job_end(feeder, &mut ans);

        if ans.pipelines.len() > 0 {
            Some(ans)
        }else{
            feeder.rewind(backup);
            None
        }
    }
}
