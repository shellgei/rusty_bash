//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::ListElem;
use crate::abst_elems::PipelineElem;
use crate::Command;
//use crate::elem_arg_delimiter::ArgDelimiter;
use nix::unistd::pipe;
use crate::scanner::*;
use crate::utils_io::set_parent_io;
use crate::abst_elems::compound;
use crate::elem_end_of_pipeline::Eop;
use crate::job::Job;

pub struct Pipeline {
    pub commands: Vec<Box<dyn PipelineElem>>,
    pub text: String,
    pub eop: Option<Eop>,
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

        conf.jobs[0] = Job::new(&self.text, &self.commands);

        if self.is_bg {
            return;
        }

        conf.jobs[0].clone().wait(conf);

        if self.not_flag {
            if conf.vars["?"] != "0" {
                conf.vars.insert("?".to_string(), "0".to_string());
            }else {
                conf.vars.insert("?".to_string(), "1".to_string());
            }
        }
    }

    fn get_text(&self) -> String { self.text.clone() }

    fn get_end(&self) -> String {
        let text = if let Some(e) = &self.eop {
            e.text.clone()
        }else{
            return "".to_string();
        };

        if text.chars().count() > 1 { 
            if text.chars().nth(0) == Some('|') && text.chars().nth(1) == Some('|') {
                return "||".to_string();
            }
            if text.chars().nth(0) == Some('&') && text.chars().nth(1) == Some('&') {
                return "&&".to_string();
            }
        }
        "".to_string()
    }   
}

impl Pipeline {
    pub fn new() -> Pipeline{
        Pipeline {
            commands: vec!(),
            text: "".to_string(),
            eop: None,
            not_flag: false,
            is_bg: false,
            job_no: 0,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();
        ans.text += &text.consume_blank();
        if text.len() > 0 {
            if text.nth(0) == '!' {
                ans.not_flag = true;
                ans.text += &text.consume(1);
            }
        }

        loop {
            ans.text += &text.consume_blank();

            let eocs;
            if let Some(mut c) = compound(text, conf) {
                eocs = c.get_eoc_string();
                ans.text += &c.get_text();
                ans.commands.push(c);
            }else if let Some(mut c) = Command::parse(text, conf) {
                eocs = c.get_eoc_string();
                ans.text += &c.text.clone();
                ans.commands.push(Box::new(c));
            }else{
                break;
            }

            if eocs != "|" {
                break;
            }


            if eocs == "|" && text.len() == 1 && text.nth(0) == '\n' {
                text.consume(1);
                if ! text.feed_additional_line(conf) {
                    return None;
                }
            }

            if scanner_end_paren(text, 0) == 1 {
                break;
            }

        }

        if let Some(eop) = Eop::parse(text) {
            if Some('&') == eop.text.chars().nth(0) 
               && Some('&') != eop.text.chars().nth(1) {
                   ans.is_bg = true;
            }

            ans.text += &eop.text.clone();
            ans.eop = Some(eop);
        }

        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}

/*
pub fn wait(child: Pid, conf: &mut ShellCore) {
    match waitpid(child, None).expect("Faild to wait child process.") {
        WaitStatus::Exited(_pid, status) => {
            conf.vars.insert("?".to_string(), status.to_string());
        }
        WaitStatus::Signaled(pid, signal, _) => {
            conf.vars.insert("?".to_string(), (128+signal as i32).to_string());
            eprintln!("Pid: {:?}, Signal: {:?}", pid, signal)
        }
        _ => {
            eprintln!("Unknown error")
        }
    };
}
*/
