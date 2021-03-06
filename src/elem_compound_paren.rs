//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::PipelineElem;
use nix::unistd::{Pid, fork, ForkResult};
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use std::process::exit;
use crate::elem_redirect::Redirect;
use crate::elem_end_of_command::Eoc;
use crate::utils_io::*;
use nix::unistd::{close, pipe};
use crate::scanner::scanner_while;

pub struct CompoundParen {
    pub script: Option<Script>,
    text: String,
    pid: Option<Pid>, 
    pub substitution_text: String,
    pub substitution: bool,
    fds: FileDescs,
    pub eoc: Option<Eoc>,
}

impl PipelineElem for CompoundParen {
    fn exec(&mut self, conf: &mut ShellCore) {
        let p = pipe().expect("Pipe cannot open");

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    self.fds.set_child_io();
                    if let Some(s) = &mut self.script {
                        if self.substitution {
                            close(p.0).expect("Can't close a pipe end");
                            dup_and_close(p.1, 1);
                        }
                        s.exec(conf);
                        close(1).expect("Can't close a pipe end");
                        exit(conf.vars["?"].parse::<i32>().unwrap());
                    };
                },
                Ok(ForkResult::Parent { child } ) => {
                    if self.substitution {
                        close(p.1).expect("Can't close a pipe end");
                        self.substitution_text  = read_pipe(p.0, child, conf)
                            .trim_end_matches('\n').to_string();
                    }
                    self.pid = Some(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
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

impl CompoundParen {
    pub fn new() -> CompoundParen{
        CompoundParen {
            script: None,
            pid: None,
            text: "".to_string(),
            substitution_text: "".to_string(),
            substitution: false,
            eoc: None,
            fds: FileDescs::new(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, substitution: bool) -> Option<CompoundParen> {
        if text.len() == 0 || text.nth(0) != '(' {
            return None;
        }

        let mut backup = text.clone();
        let mut ans = CompoundParen::new();
        let mut input_success;

        loop{
            text.consume(1);
            if let Some(s) = Script::parse(text, conf, vec!(")")) {
                ans.text = "(".to_owned() + &s.text + ")";
                ans.script = Some(s);
            }else{
                (backup, input_success) = text.rewind_feed_backup(&backup, conf);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
                continue;
            }

            if text.len() == 0 || text.nth(0) != ')' {
                (backup, input_success) = text.rewind_feed_backup(&backup, conf);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
            }else{
                break;
            }
        }

        text.consume(1);
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
