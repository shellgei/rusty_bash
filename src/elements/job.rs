//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::operators::*;
use crate::elements::command::CommandType;
use crate::elements::pipeline::Pipeline;
use crate::utils::blue_string;
use crate::core::proc;
use nix::unistd::{ForkResult};
use nix::unistd;
use std::process::exit;

#[derive(Debug)]
pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub pipeline_ends: Vec<ControlOperator>,
    pub text: String,
    pub is_bg: bool,
}

impl Job {
    pub fn exec(&mut self, conf: &mut ShellCore) {
        conf.add_job_entry(self);

        if self.is_bg && 
            (self.pipeline_ends[0] == ControlOperator::And || 
             self.pipeline_ends[0] == ControlOperator::Or) {
               self.exec_and_or_bg_job(conf);
               return;
           }

        self.exec_job(conf);

        /*
        let mut eop = ControlOperator::NoChar;
        for (i, p) in self.pipelines.iter_mut().enumerate() {
            if conf.has_flag('d') {
                eprintln!("{}", blue_string(&p.get_text()));
            }

            let status = conf.get_var("?") == "0";
           
            if (status && eop == ControlOperator::Or) || (!status && eop == ControlOperator::And) {
                eop = self.pipeline_ends[i].clone();
                continue;
            }
            p.exec(conf);
            eop = self.pipeline_ends[i].clone();
        }*/
    }

    fn exec_job(&mut self, conf: &mut ShellCore) {
        let mut eop = ControlOperator::NoChar;
        for (i, p) in self.pipelines.iter_mut().enumerate() {
            if conf.has_flag('d') {
                eprintln!("{}", blue_string(&p.get_text()));
            }

            let status = conf.get_var("?") == "0";
           
            if (status && eop == ControlOperator::Or) || (!status && eop == ControlOperator::And) {
                eop = self.pipeline_ends[i].clone();
                continue;
            }
            p.exec(conf);
            eop = self.pipeline_ends[i].clone();
        }
    }

    fn exec_and_or_bg_job(&mut self, conf: &mut ShellCore) {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                proc::set_signals();
                let pid = nix::unistd::getpid();
                let _ = unistd::setpgid(pid, pid);

               // eprintln!("HERE");
                self.exec_job(conf);

                exit(conf.vars["?"].parse::<i32>().unwrap());
            },
            Ok(ForkResult::Parent { child } ) => {
                //self.pid = Some(child);
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

    fn is_end_condition(parent: &CommandType, op: &ControlOperator) -> bool {
        ( op == &ControlOperator::Semicolon || op == &ControlOperator::BgAnd ) ||
        ( parent == &CommandType::Paren && op == &ControlOperator::RightParen ) ||
        ( parent == &CommandType::Case && op == &ControlOperator::DoubleSemicolon )
    }

    fn set_pipelineend(text: &mut Feeder, ans: &mut Job, parent_type: &CommandType) -> bool {
        let (n, op) = text.scanner_control_op();
        if let Some(p) = op {
            if &p == &ControlOperator::Semicolon || &p == &ControlOperator::BgAnd {
                ans.text += &text.consume(n);
            }
            ans.pipeline_ends.push(p.clone());
            if Job::is_end_condition(parent_type, &p) {
                return true;
            }

            ans.text += &text.consume(n);
        }else{
            ans.pipeline_ends.push(ControlOperator::NoChar);
        }

        false
    }

    fn read_blank(text: &mut Feeder, ans: &mut Job) {
        loop {
            let before = ans.text.len();
            ans.text += &text.consume_blank_return();
            ans.text += &text.consume_comment();

            if before == ans.text.len() || text.len() == 0 {
                return;
            }
        }
    }

    pub fn parse_elem(text: &mut Feeder, conf: &mut ShellCore, ans: &mut Job, parent_type: &CommandType) -> bool {
        let mut go_next = true;

        if let Some(result) = Pipeline::parse(text, conf) {
            ans.text += &result.text;
            ans.pipelines.push(result);

            if Job::set_pipelineend(text, ans, parent_type){
                go_next = false;
            }
        }
        else {
            go_next = false;
        }

        go_next
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, parent_type: &CommandType) -> Option<Job> {
        if text.len() == 0 {
            return None;
        };
    
        let backup = text.clone();

        let mut ans = Job::new();
        Job::read_blank(text, &mut ans);
        while  Job::parse_elem(text, conf, &mut ans, parent_type) {
            if text.len() == 0 && parent_type == &CommandType::Null {
                break;
            }

            text.request_next_line(conf);
            Job::read_blank(text, &mut ans);
        }

        if ans.pipelines.len() > 0 {
            if ans.pipeline_ends.last().unwrap() == &ControlOperator::BgAnd {
                ans.is_bg = true;
            }
//            eprintln!("{:?}", &ans);
            Some(ans)
        }else{
            text.rewind(backup);
            None
        }
    }
}
