//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::PipelineElem;
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use crate::elem_redirect::Redirect;
use crate::elem_end_of_command::Eoc;
use crate::utils_io::*;
use std::process::exit;

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
    pub script: Script,
    text: String,
    pid: Option<Pid>, 
    pub substitution_text: String,
    pub eoc: Option<Eoc>,
    fds: FileDescs,
}

impl PipelineElem for CompoundBrace {
    fn exec_elems(&mut self, conf: &mut ShellCore) {
             self.script.exec(conf);
             if ! self.fds.no_connection() {
                 exit(conf.vars["?"].parse::<i32>().unwrap());
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

impl CompoundBrace {
    pub fn new(script: Script) -> CompoundBrace{
        CompoundBrace {
            script: script,
            pid: None,
            text: "".to_string(),
            substitution_text: "".to_string(),
            fds: FileDescs::new(),
            eoc: None,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundBrace> {
        if text.len() == 0 || text.nth(0) != '{' {
            return None;
        }

        let mut backup = text.clone();
        let mut ans;
        let mut input_success;

        loop {
            text.consume(1);
            if let Some(s) = Script::parse(text, conf, vec!("}")) {
                if ! tail_check(&s.text){
                    text.rewind(backup);
                    return None;
                }
    
                let text = "{".to_owned() + &s.text.clone() + "}";
                ans = CompoundBrace::new(s);
                ans.text = text;
            }else{
                (backup, input_success) = text.rewind_feed_backup(&backup, conf);
                if ! input_success {
                    eprintln!("ESC");
                    text.consume(text.len());
                    return None;
                }
                continue;
            }
    
            if text.len() == 0 || text.nth(0) != '}' {
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
