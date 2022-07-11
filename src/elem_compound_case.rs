//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::PipelineElem;
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use crate::elem_redirect::Redirect;
use nix::unistd::Pid;
use crate::utils_io::*;
use crate::elem_end_of_command::Eoc;
use crate::scanner::*;
use crate::elem_arg::Arg;
use crate::bash_glob::glob_match;
use crate::abst_elems::CommandElem;

pub struct CompoundCase {
    pub arg: Arg,
    pub conddo: Vec<(String, Option<Script>)>,
    text: String,
    pid: Option<Pid>,
    fds: FileDescs,
    pub eoc: Option<Eoc>,
}

impl PipelineElem for CompoundCase {
    fn get_pid(&self) -> Option<Pid> { self.pid }
    fn set_pid(&mut self, pid: Pid) { self.pid = Some(pid); }
    fn no_connection(&self) -> bool { self.fds.no_connection() }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn set_child_io(&self){
        self.fds.set_child_io();
    }

    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }

    fn get_eoc_string(&mut self) -> String {
        if let Some(e) = &self.eoc {
            return e.text.clone();
        }

        "".to_string()
    }

    fn get_text(&self) -> String { self.text.clone() }

    fn exec_elems(&mut self, conf: &mut ShellCore) {
        let arg_str = self.arg.eval(conf).join(" ");

        for (cond, doing) in &mut self.conddo {
            if glob_match(cond, &arg_str) {
                if let Some(d) = doing {
                    d.exec(conf);
                }
                break;
            }
        }
    }
}

impl CompoundCase {
    pub fn new(arg: Arg) -> CompoundCase{
        CompoundCase {
            arg: arg, 
            conddo: vec!(),
            text: "".to_string(),
            fds: FileDescs::new(),
            pid: None,
            eoc: None,
        }
    }


    fn parse_cond_do_pair(text: &mut Feeder, conf: &mut ShellCore, ans: &mut CompoundCase) -> bool {
        CompoundCase::next_line(text, conf, ans);

        let pos = scanner_until_escape(text, 0, ")");

        if pos == 0 || pos == text.len() {
            return false;
        }

        let cond = text.consume(pos);
        ans.text += &cond.clone();
        ans.text += &text.consume(1);

        CompoundCase::next_line(text, conf, ans);

        let doing = if text.len() >= 2 && text.compare(0, ";;") {
            None
        }else if let Some(s) = Script::parse(text, conf) {
            ans.text += &s.text;
            Some(s)
        }else{
            return false;
        };

        CompoundCase::next_line(text, conf, ans);

        if text.len() >= 2 && text.compare(0, ";;") {
            ans.text += &text.consume(2);
        }/*else{
            return false;
        }*/

        ans.conddo.push( (cond, doing) );
        true
    }

    fn next_line(text: &mut Feeder, conf: &mut ShellCore, ans: &mut CompoundCase) -> bool {
        ans.text += &text.consume_blank();

        if text.len() == 0 || text.nth(0) == '\n' {
            if ! text.feed_additional_line(conf){
                return false;
            }
        }
        true
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundCase> {
        if text.len() < 4 || ! text.compare(0, "case") {
            return None;
        }

        let backup = text.clone();
        let ans_text = text.consume(4) + &text.consume_blank();

        let arg = if let Some(a) = Arg::parse(text, conf, false, false) {
            a
        }else{
            text.rewind(backup);
            return None;
        };

        let mut ans = CompoundCase::new(arg);
        ans.text = ans_text;

        ans.text += &text.consume_blank();

        if text.len() >= 2 && text.compare(0, "in") {
            ans.text += &text.consume(2);
        }else{
            text.rewind(backup);
            return None;
        }

        loop {
            ans.text += &text.consume_blank_return();
            CompoundCase::next_line(text, conf, &mut ans);
            ans.text += &text.consume_blank_return();

            if text.len() >= 4 && text.compare(0, "esac") {
                ans.text += &text.consume(4);
                break;
            }

            if ! CompoundCase::parse_cond_do_pair(text, conf, &mut ans) {
                text.rewind(backup);
                return None;
            }
        }

        loop {
            ans.text += &text.consume_blank();

            if let Some(r) = Redirect::parse(text){
                ans.text += &r.text;
                ans.fds.redirects.push(Box::new(r));
            }else{
                break;
            }
        }
        if let Some(e) = Eoc::parse(text){
            ans.text += &e.text;
            ans.eoc = Some(e);
        }

        if ans.conddo.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
