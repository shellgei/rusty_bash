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
use crate::scanner::scanner_while;

/* ( script ) */
pub struct CompoundCase {
    pub conddo: Option<(Script, Script)>,
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
        loop {
            if let Some((cond, doing)) = &mut self.conddo {
                cond.exec(conf);
                if conf.vars["?"] != "0" {
                    conf.vars.insert("?".to_string(), "0".to_string());
                    break;
                }
                doing.exec(conf);
            }
        }
    }
}

impl CompoundCase {
    pub fn new() -> CompoundCase{
        CompoundCase {
            conddo: None,
            text: "".to_string(),
            fds: FileDescs::new(),
            pid: None,
            eoc: None,
        }
    }


    fn parse_cond_do_pair(text: &mut Feeder, conf: &mut ShellCore, ans: &mut CompoundCase) -> bool {
        CompoundCase::next_line(text, conf, ans);

        let cond = if let Some(s) = Script::parse(text, conf) {
            ans.text += &s.text;
            s
        }else{
            return false;
        };

        CompoundCase::next_line(text, conf, ans);

        if text.compare(0, "do"){
            ans.text += &text.consume(2);
        }

        CompoundCase::next_line(text, conf, ans);

        let doing = if let Some(s) = Script::parse(text, conf) {
            ans.text += &s.text;
            s
        }else{
            return false;
        };

        CompoundCase::next_line(text, conf, ans);

        ans.conddo = Some( (cond, doing) );
        true
    }

    fn next_line(text: &mut Feeder, conf: &mut ShellCore, ans: &mut CompoundCase) -> bool {
        let d = scanner_while(text, 0, " \t");
        ans.text += &text.consume(d);

        if text.len() == 0 || text.nth(0) == '\n' {
            if ! text.feed_additional_line(conf){
                return false;
            }
        }
        true
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundCase> {
        if text.len() < 5 || ! text.compare(0, "while") {
            return None;
        }

        let backup = text.clone();

        let mut ans = CompoundCase::new();
        ans.text += &text.consume(5);

        if ! CompoundCase::parse_cond_do_pair(text, conf, &mut ans) {
            text.rewind(backup);
            return None;
        }

        if text.compare(0, "done"){
            ans.text += &text.consume(4);
        }else{
            text.rewind(backup);
            return None;
        }

        loop {
            let d = scanner_while(text, 0, " \t");
            ans.text += &text.consume(d);

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
