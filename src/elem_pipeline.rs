//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::ListElem;
use crate::abst_elems::PipelineElem;
use crate::Command;
use crate::element_list::ControlOperator;
use nix::unistd::pipe;
use crate::scanner::*;
use crate::utils_io::set_parent_io;
use crate::abst_elems::compound;
use crate::job::Job;

pub struct Pipeline {
    pub commands: Vec<Box<dyn PipelineElem>>,
    pub text: String,
    pub is_bg: bool,
    pub job_no: u32,
    not_flag: bool,
}

impl ListElem for Pipeline {
    fn exec(&mut self, conf: &mut ShellCore) {
        let len = self.commands.len();
        let mut prevfd = -1;
        for (i, c) in self.commands.iter_mut().enumerate() {
            let mut p = (-1, -1);
            if i != len-1 {
                p = pipe().expect("Pipe cannot open");
            };
            c.set_pipe(p.0, p.1, prevfd);
            c.exec(conf);
            set_parent_io(c.get_pipe_out());
            prevfd = c.get_pipe_end();
        }


        if self.is_bg {
            conf.jobs.push(Job::new(&self.text, &self.commands));
            return;
        }else{
            conf.jobs[0] = Job::new(&self.text, &self.commands);
        }

        let pipestatus = conf.jobs[0].clone().wait();
        if let Some(s) = pipestatus.last() {
            conf.set_var("?", s);
            conf.set_var("PIPESTATUS", &pipestatus.join(" "));
        }

        if self.not_flag {
            if conf.vars["?"] != "0" {
                conf.set_var("?", "0");
            }else {
                conf.set_var("?", "1");
            }
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl Pipeline {
    pub fn new() -> Pipeline{
        Pipeline {
            commands: vec![],
            text: "".to_string(),
            not_flag: false,
            is_bg: false,
            job_no: 0,
        }
    }

    pub fn set_control_op(text: &mut Feeder, ans: &mut Pipeline) {
        let (n, op) = scanner_control_op(text);
        if let Some(p) = op {
            if p == ControlOperator::Pipe || p == ControlOperator::PipeAnd {
                ans.text += &text.consume(n);
            }
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();
        ans.text += &text.consume_blank();
        if text.compare(0, "!") {
            ans.not_flag = true;
            ans.text += &text.consume(1);
        }

        loop {
            ans.text += &text.consume_blank();

            let op;
            if let Some(c) = compound(text, conf) {
                ans.text += &c.get_text();
                ans.commands.push(c);
                (_, op) = scanner_control_op(text);
                Pipeline::set_control_op(text, &mut ans);
            }else if let Some(c) = Command::parse(text, conf) {
                ans.text += &c.text.clone();
                ans.commands.push(Box::new(c));
                (_, op) = scanner_control_op(text);
                Pipeline::set_control_op(text, &mut ans);
            }else{
                while text.compare(0, "\n") {
                    ans.text += &text.consume(1); 
                }
                break;
            }

            if let Some(p) = op {
                if p == ControlOperator::BgAnd {
                    ans.is_bg = true;
                }
                if p != ControlOperator::Pipe && p != ControlOperator::PipeAnd {
                    break;
                }
            }

            if text.compare(0, "\n") {
                text.consume(1);
                if ! text.feed_additional_line(conf) {
                    return None;
                }
            }

            if scanner_end_paren(text, 0) == 1 {
                break;
            }
        }

        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
