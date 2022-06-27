//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid, fork, ForkResult};
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use crate::elem_redirect::Redirect;
use crate::elem_end_of_command::Eoc;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::utils_io::*;
use std::process::exit;

/* ( script ) */
pub struct CompoundIf {
    pub ifthen: Vec<(Script, Script)>,
    pub else_do: Option<Script>,
    /*
    pub script: Script,
    pub redirects: Vec<Box<Redirect>>,
    pub text: String,
    pid: Option<Pid>, 
    pub pipein: RawFd,
    pub pipeout: RawFd,
    /* The followings are set by a pipeline.  */
    pub substitution_text: String,
    pub prevpipein: RawFd,
    pub eoc: Option<Eoc>,
    */
}

impl ScriptElem for CompoundIf {
    fn exec(&mut self, conf: &mut ShellCore) {
        /*
        if self.pipeout == -1 && self.pipein == -1 && self.prevpipein == -1 
            && self.redirects.len() == 0 /* && self.script.args_for_function.len() == 0 */ {
             self.script.exec(conf);
             return;
        };

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    set_child_io(self.pipein, self.pipeout, self.prevpipein, &self.redirects);
                    self.script.exec(conf);
                    exit(conf.vars["?"].parse::<i32>().unwrap());
                },
                Ok(ForkResult::Parent { child } ) => {
                    self.pid = Some(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
        */
    }

    /*
    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.pipein = pin;
        self.pipeout = pout;
        self.prevpipein = pprev;
    }

    fn get_pipe_end(&mut self) -> RawFd { self.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.pipeout }

    fn get_eoc_string(&mut self) -> String {
        if let Some(e) = &self.eoc {
            return e.text.clone();
        }

        "".to_string()
    }
    */
}

impl CompoundIf {
    /*
    pub fn new(script: Script) -> CompoundIf{
        CompoundIf {
            script: script,
            pid: None,
            redirects: vec!(),
            text: "".to_string(),
            substitution_text: "".to_string(),
            pipein: -1,
            pipeout: -1,
            prevpipein: -1,
            eoc: None,
        }
    }*/

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundIf> {
        if text.len() < 2 || ! text.compare(0, "if".to_string()) {
            return None;
        }

        None
        /*
        if text.len() == 0 || text.nth(0) != '{' {
            return None;
        }

        let mut backup = text.clone();
        let mut ans;
        let mut input_success;

        loop {
            text.consume(1);
            if let Some(s) = Script::parse(text, conf, true) {
                //eprintln!("script: {}", s.text);
                if ! tail_check(&s.text){
                    text.rewind(backup); return None; }
    
                let text = "{".to_owned() + &s.text.clone() + "}";
                ans = CompoundIf::new(s);
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
        */
    }
}
