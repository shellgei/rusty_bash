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

/* ( script ) */
pub struct CompoundIf {
    pub ifthen: Vec<(Script, Script)>,
    pub else_do: Option<Script>,
    text: String,
    pid: Option<Pid>,
    pub eoc: Option<Eoc>,
    fds: FileDescs,
}

impl PipelineElem for CompoundIf {
    fn exec_elems(&mut self, conf: &mut ShellCore) {
        for pair in self.ifthen.iter_mut() {
             pair.0.exec(conf);
             if conf.vars["?"] != "0" {
                continue;
             }
             pair.1.exec(conf);
             return;
        }

        if let Some(s) = &mut self.else_do {
            s.exec(conf);
        }
    }

    fn set_pid(&mut self, pid: Pid) { self.pid = Some(pid); }
    fn no_connection(&self) -> bool { self.fds.no_connection() }

    fn set_child_io(&self){
        self.fds.set_child_io();
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
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
}

impl CompoundIf {
    pub fn new() -> CompoundIf{
        CompoundIf {
            ifthen: vec!(),
            else_do: None,
            fds: FileDescs::new(),
            text: "".to_string(),
            pid: None,
            eoc: None,
        }
    }


    fn parse_if_then_pair(text: &mut Feeder, conf: &mut ShellCore, ans: &mut CompoundIf) -> bool {
        ans.text += &text.request_next_line(conf);

        let cond = if let Some(s) = Script::parse(text, conf, vec!("then")) {
            ans.text += &s.text;
            s
        }else{
            return false;
        };

        ans.text += &text.request_next_line(conf);

        if text.compare(0, "then"){
            ans.text += &text.consume(4);
        }

        ans.text += &text.request_next_line(conf);

        let doing = if let Some(s) = Script::parse(text, conf, ["elif", "else"].to_vec()) {
            ans.text += &s.text;
            s
        }/*else if let Some(s) = Script::parse(text, conf, "else") {
            ans.text += &s.text;
            s
        }*/else{
            return false;
        };

        ans.text += &text.request_next_line(conf);

        ans.ifthen.push( (cond, doing) );
        true
    }

    fn parse_else_fi(text: &mut Feeder, conf: &mut ShellCore, ans: &mut CompoundIf) -> bool {
        //CompoundIf::next_line(text, conf, ans);
        ans.text += &text.request_next_line(conf);
        

        ans.else_do = if let Some(s) = Script::parse(text, conf, vec!("fi")) {
            ans.text += &s.text;
            Some(s)
        }else{
            return false;
        };

        ans.text += &text.request_next_line(conf);

        if text.compare(0, "fi"){
             ans.text += &text.consume(2);
        }else{
             return false;
        }

        true
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundIf> {
        if text.len() < 2 || ! text.compare(0, "if") {
            return None;
        }

        let backup = text.clone();

        let mut ans = CompoundIf::new();
        ans.text += &text.consume(2);

        //eprintln!("REM: '{}'", text._text());
        loop {
            if ! CompoundIf::parse_if_then_pair(text, conf, &mut ans) {
                text.rewind(backup);
                return None;
            }
    
            if text.compare(0, "fi"){
                ans.text += &text.consume(2);
                break;
            }else if text.compare(0, "elif"){
                ans.text += &text.consume(4);
                continue;
            }else if text.compare(0, "else"){
                ans.text += &text.consume(4);
                if CompoundIf::parse_else_fi(text, conf, &mut ans) {
                    break;
                }
            }

            text.rewind(backup);
            return None;
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

        Some(ans)
    }
}
