//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid, fork, ForkResult, close};
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use std::process::exit;
use crate::elem_redirect::Redirect;
use crate::elem_end_of_command::Eoc;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::utils_io::*;

fn tail_check(s: &String) -> bool{
    for ch in s.chars().rev() {
        match ch {
            ' ' => continue,
            '\n' => return true,
            ';' => return true,
            '\t' => return true,
            _ => return false,
        }
    }
    false
}

/* ( script ) */
pub struct CompoundBrace {
    pub script: Option<Script>,
    pub redirects: Vec<Box<Redirect>>,
    pub text: String,
    pid: Option<Pid>, 
    pub pipein: RawFd,
    pub pipeout: RawFd,
    /* The followings are set by a pipeline.  */
    pub prevpipein: RawFd,
    pub eoc: Option<Eoc>,
}

impl ScriptElem for CompoundBrace {
    fn exec(&mut self, conf: &mut ShellCore) {
        if self.pipeout == -1 && self.pipein == -1 && self.prevpipein == -1 && self.redirects.len() == 0 {
            if let Some(s) = &mut self.script {
                s.exec(conf);
                return;
            };
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    set_child_io(self.pipein, self.pipeout, self.prevpipein, &self.redirects);
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

    fn set_parent_io(&mut self) {
        if self.pipeout >= 0 {
            close(self.pipeout).expect("Cannot close outfd");
        };
    }

    fn get_pipe_end(&mut self) -> RawFd { self.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.pipeout }

    fn get_eoc_string(&mut self) -> String {
        if let Some(e) = &self.eoc {
            return e.text.clone();
        }

        "".to_string()
    }
}

impl CompoundBrace {
    pub fn new() -> CompoundBrace{
        CompoundBrace {
            script: None,
            pid: None,
            redirects: vec!(),
            text: "".to_string(),
            pipein: -1,
            pipeout: -1,
            prevpipein: -1,
            eoc: None,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundBrace> {
        if text.len() == 0 || text.nth(0) != '{' {
            return None;
        }

        let mut backup = text.clone();
        let mut ans = CompoundBrace::new();

        loop {
            text.consume(1);
            if let Some(s) = Script::parse(text, conf, true) {
                //eprintln!("script: {}", s.text);
                if ! tail_check(&s.text){
                    text.rewind(backup);
                    return None;
                }
    
                ans.text = "{".to_owned() + &s.text + "}";
                ans.script = Some(s);
            }else{
                backup = text.rewind_feed_backup(&backup, conf);
                continue;
            }
    
            if text.len() == 0 || text.nth(0) != '}' {
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
