//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid, fork, ForkResult, pipe};
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use std::process::exit;
use crate::utils_io::dup_and_close;
use crate::elem_redirect::Redirect;
use crate::elem_end_of_command::Eoc;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::utils_io::*;

/* ( script ) */
pub struct CompoundParen {
    pub script: Option<Script>,
    pub redirects: Vec<Box<Redirect>>,
    pub text: String,
    pid: Option<Pid>, 
    pub pipein: RawFd,
    pub pipeout: RawFd,
    /* The followings are set by a pipeline or a com expansion. */
    pub expansion: bool,
    pub expansion_str: String,
    pub prevpipein: RawFd,
    pub eoc: Option<Eoc>,
}

impl ScriptElem for CompoundParen {
    fn exec(&mut self, conf: &mut ShellCore) {
        if self.expansion {
            self.set_command_expansion_pipe();
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    if self.expansion {
                        dup_and_close(self.pipeout, 1);
                    }else{
                        set_child_io(self.pipein, self.pipeout, self.prevpipein, &self.redirects);
                       // self.set_child_io();
                    }
                    if let Some(s) = &mut self.script {
                        s.exec(conf);
                        exit(conf.vars["?"].parse::<i32>().unwrap());
                    };
                },
                Ok(ForkResult::Parent { child } ) => {
                    self.pid = Some(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.pipein = pin;
        self.pipeout = pout;
        self.prevpipein = pprev;
    }

    /*
    fn set_parent_io(&mut self) {
        if self.pipeout >= 0 {
            close(self.pipeout).expect("Cannot close outfd");
        }
//        return self.pipein;
    }
    */

    fn get_pipe_end(&mut self) -> RawFd { self.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.pipeout }

    fn get_eoc_string(&mut self) -> String {
        if let Some(e) = &self.eoc {
            return e.text.clone();
        }

        "".to_string()
    }
}

impl CompoundParen {
    pub fn new() -> CompoundParen{
        CompoundParen {
            script: None,
            pid: None,
            redirects: vec!(),
            text: "".to_string(),
            pipein: -1,
            pipeout: -1,
            expansion: false,
            expansion_str: "".to_string(),
            prevpipein: -1,
            eoc: None,
        }
    }

    fn set_command_expansion_pipe(&mut self){
        let p = pipe().expect("Pipe cannot open");
        self.pipein = p.0;
        self.pipeout = p.1;
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundParen> {
        if text.len() == 0 || text.nth(0) != '(' {
            return None;
        }

        let mut backup = text.clone();
        let mut ans = CompoundParen::new();

        loop{
            text.consume(1);
            if let Some(s) = Script::parse(text, conf, true) {
                ans.text = "(".to_owned() + &s.text + ")";
                ans.script = Some(s);
            }else{
                backup = text.rewind_feed_backup(&backup, conf);
                continue;
            }

            if text.len() == 0 || text.nth(0) != ')' {
                backup = text.rewind_feed_backup(&backup, conf);
            }else{
                break;
            }
        }

        text.consume(1);

        loop {
            if let Some(d) = ArgDelimiter::parse(text){
                ans.text += &d.text;
            }

            if let Some(r) = Redirect::parse(text){
                    ans.text += &r.text;
                    ans.redirects.push(Box::new(r));
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
