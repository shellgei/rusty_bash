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
    pub text: String,
    pub redirects: Vec<Box<Redirect>>,
    pub pipein: RawFd,
    pub pipeout: RawFd,
    pub prevpipein: RawFd,
//    pub eoc: Option<Eoc>,
}

impl ScriptElem for CompoundIf {
    fn exec(&mut self, conf: &mut ShellCore) {
        for pair in self.ifthen.iter_mut() {
             pair.0.exec(conf);
             if conf.vars["?"] != "0" {
                conf.vars.insert("?".to_string(), "0".to_string());
                return;
             }

             pair.1.exec(conf);
        }
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
    pub fn new() -> CompoundIf{
        CompoundIf {
            ifthen: vec!(),
            else_do: None,
            redirects: vec!(),
            text: "".to_string(),
            pipein: -1,
            pipeout: -1,
            prevpipein: -1,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundIf> {
        if text.len() < 2 || ! text.compare(0, "if".to_string()) {
            return None;
        }

        let backup = text.clone();

        let mut ans = CompoundIf::new();
        ans.text += &text.consume(2);

        let cond = if let Some(s) = Script::parse(text, conf, true) {
            ans.text += &s.text;
            s
        }else{
            text.rewind(backup);
            return None;
        };

        if let Some(d) = ArgDelimiter::parse(text){
            ans.text += &d.text;
        }

        if text.compare(0, "then".to_string()){
            ans.text += &text.consume(4);
        }

        let doing = if let Some(s) = Script::parse(text, conf, true) {
            ans.text += &s.text;
            s
        }else{
            text.rewind(backup);
            return None;
        };

        if text.compare(0, "fi".to_string()){
            ans.text += &text.consume(2);
        }else{
            text.rewind(backup);
            return None;
        }

        if let Some(d) = ArgDelimiter::parse(text){
            ans.text += &d.text;
        }

        ans.ifthen.push((cond, doing));
        Some(ans)
    }
}
