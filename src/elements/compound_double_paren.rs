//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::Compound;
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::elements::redirect::Redirect;
use crate::file_descs::*;
//use crate::feeder::scanner::*;
use crate::calculator::calculate;

pub struct CompoundDoubleParen {
    text: String,
    expression: String,
    pid: Option<Pid>, 
    pub substitution_text: String,
    pub substitution: bool,
    fds: FileDescs,
//    pub eoc: Option<Eoc>,
}

impl Compound for CompoundDoubleParen {
    fn exec(&mut self, conf: &mut ShellCore) {
        self.substitution_text = calculate(self.expression.clone(), conf);

        let status = if self.substitution_text == "0" {
            "1"
        }else{
            "0"
        }.to_string();

        conf.set_var("?", &status);
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
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
            //eoc: None,
            fds: FileDescs::new(),
        }
    }

    // TODO: this function must parse ((1+$(echo a | wc -l)) for example. 
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, substitution: bool) -> Option<CompoundDoubleParen> {
        if text.len() < 2 || ! text.starts_with( "((") {
            return None;
        }

        let mut backup = text.clone();
        let mut ans = CompoundDoubleParen::new();
        let mut input_success;

        loop{
            ans.text = text.consume(2);

            let pos = text.scanner_until(0, ")");

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

            if text.len() < 2 || ! text.starts_with( "))") {
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
            //let d = text.scanner_blank();
            ans.text += &text.consume_blank();

            if let Some(r) = Redirect::parse(text, conf){
                    ans.text += &r.text;
                    ans.fds.redirects.push(Box::new(r));
            }else{
                break;
            }
        }

        Some(ans)
    }
}
