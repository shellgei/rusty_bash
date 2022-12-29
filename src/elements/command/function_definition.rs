//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
//use crate::feeder::scanner::*;
use crate::elements::command;
use crate::elements::command::Command;

use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::FileDescs;

pub struct FunctionDefinition {
    pub name: String,
    pub body: Box<dyn Command>,
    pid: Option<Pid>, 
    pub text: String,
    fds: FileDescs,
    session_leader: bool,
}

impl Command for FunctionDefinition {
    fn exec_elems(&mut self, conf: &mut ShellCore) {
        conf.functions.insert(self.name.clone(), self.body.get_text());
    }
    fn set_pid(&mut self, pid: Pid) { self.pid = Some(pid); }
    fn set_session_leader(&mut self) { self.session_leader = true; }
    fn no_connection(&self) -> bool { self.fds.no_connection() }

    fn set_child_io(&mut self, conf: &mut ShellCore) -> Result<(), String> {
        self.fds.set_child_io(conf)
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

impl FunctionDefinition {
    pub fn new(name: String, body: Box<dyn Command>, text: String) -> FunctionDefinition{
        FunctionDefinition {
            name: name,
            body: body,
            text: text,
            pid: None,
            fds: FileDescs::new(),
            session_leader: false,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<FunctionDefinition> {
         let backup = text.clone();
         let mut ans_text = String::new();

         if text.starts_with("function") {
            ans_text += &text.consume(8);
            ans_text += &text.consume_blank();
         }

         let var_pos = text.scanner_name(0);
         if var_pos == 0 {
             text.rewind(backup);
             return None;
         }
         let name = text.consume(var_pos);
         ans_text += &text.consume_blank();


         if ! text.starts_with("(") {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
         ans_text += &text.consume_blank();
 
         if ! text.starts_with(")") {
             text.rewind(backup);
             return None;
         }
         ans_text += &text.consume(1);
         ans_text += &text.consume_blank();
 
         if let Some(c) = command::parse(text, conf){
//             conf.functions.insert(name.clone(), c.get_text());
             Some( FunctionDefinition::new(name, c, ans_text) )
         }else{
             text.rewind(backup);
             None
         }
    }
}
