//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use crate::operators::ControlOperator;
use nix::unistd::pipe;
use crate::file_descs::FileDescs;
use crate::elements::command;
use crate::core::job::Job;

pub struct Pipeline {
    pub commands: Vec<Box<dyn Command>>,
    pub text: String,
    pub is_bg: bool,
    pub job_no: u32,
    not_flag: bool,
}

impl Pipeline {
    pub fn exec(&mut self, core: &mut ShellCore) {
        let len = self.commands.len();
        let mut prevfd = -1;
        for (i, c) in self.commands.iter_mut().enumerate() {
            let mut p = (-1, -1);
            if i != len-1 {
                p = pipe().expect("Pipe cannot open");
            };
            c.set_pipe(p.0, p.1, prevfd);
            c.exec(core);
            FileDescs::set_parent_io(c.get_pipe_out());
            prevfd = c.get_pipe_end();
        }

        core.jobs.push(Job::new(&self.text, &self.commands));
        if self.is_bg {
            return;
        }else{
            core.fg_job = core.jobs.len() - 1;
        //    core.jobs[0] = Job::new(&self.text, &self.commands);
        }
        core.wait_job(core.fg_job);

        if self.not_flag {
            if core.vars["?"] != "0" {
                core.set_var("?", "0");
            }else {
                core.set_var("?", "1");
            }
        }
    }

    pub fn get_text(&self) -> String { self.text.clone() }

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
        let (n, op) = text.scanner_control_op();
        if let Some(p) = op {
            if p == ControlOperator::Pipe || p == ControlOperator::PipeAnd {
                ans.text += &text.consume(n);
            }
        }
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();
        ans.text += &text.consume_blank();
        if text.starts_with( "!") {
            ans.not_flag = true;
            ans.text += &text.consume(1);
        }

        loop {
            ans.text += &text.consume_blank();

            let op;
            if let Some(c) = command::parse(text, core) {
                ans.text += &c.get_text();
                ans.commands.push(c);
                (_, op) = text.scanner_control_op();
                Pipeline::set_control_op(text, &mut ans);
            }else{
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

            if text.starts_with( "\n") {
                text.consume(1);
                if ! text.feed_additional_line(core) {
                    return None;
                }
            }

            if text.starts_with(")") {
            //if scanner_end_paren(text, 0) == 1 {
                break;
            }
        }

        ans.text += &text.consume_blank_return();
        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
