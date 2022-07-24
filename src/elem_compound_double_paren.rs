//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::PipelineElem;
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::elem_redirect::Redirect;
use crate::elem_end_of_command::Eoc;
use crate::utils_io::*;
use crate::scanner::*;

pub struct CompoundDoubleParen {
    text: String,
    expression: String,
    pid: Option<Pid>, 
    pub substitution_text: String,
    pub substitution: bool,
    fds: FileDescs,
    pub eoc: Option<Eoc>,
}

impl PipelineElem for CompoundDoubleParen {
    fn exec(&mut self, conf: &mut ShellCore) {
        eprintln!("{}", self.expression);
        self.substitution_text = self.expression.clone();

        let status = if self.substitution_text == "0" {
            "1"
        }else{
            "0"
        }.to_string();

        conf.vars.insert("?".to_string(), status);
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

impl CompoundDoubleParen {
    pub fn new() -> CompoundDoubleParen{
        CompoundDoubleParen {
           // script: None,
            pid: None,
            text: "".to_string(),
            expression: "".to_string(),
            substitution_text: "".to_string(),
            substitution: false,
            eoc: None,
            fds: FileDescs::new(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, substitution: bool) -> Option<CompoundDoubleParen> {
        if text.len() < 2 || ! text.compare(0, "((") {
            return None;
        }

        let mut backup = text.clone();
        let mut ans = CompoundDoubleParen::new();
        let mut input_success;

        loop{
            ans.text = text.consume(2);

            let pos = scanner_until(text, 0, ")");

            /*
            if let Some(s) = Script::parse(text, conf, vec!(")")) {
                ans.text = "((".to_owned() + &s.text + "))";
                ans.script = Some(s);
                */
            if pos != text.len() {
                ans.expression = text.consume(pos);
                ans.text += &ans.expression.clone();
            }else{
                (backup, input_success) = text.rewind_feed_backup(&backup, conf);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
                continue;
            }

            if text.len() < 2 || ! text.compare(0, "))") {
                (backup, input_success) = text.rewind_feed_backup(&backup, conf);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
            }else{
                break;
            }
        }

        text.consume(2);
        if substitution {
            return Some(ans);
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
