//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
//use crate::feeder::scanner::*;
use crate::elements::abst_command;
use crate::elements::abst_command::AbstCommand;

use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::FileDescs;

pub struct Function {
    pub name: String,
    pub body: Box<dyn AbstCommand>,
    pid: Option<Pid>, 
    pub text: String,
    fds: FileDescs,
}

impl AbstCommand for Function {
    fn exec_elems(&mut self, _: &mut ShellCore) {}
    fn set_pid(&mut self, pid: Pid) { self.pid = Some(pid); }
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

impl Function {
    pub fn new(name: String, body: Box<dyn AbstCommand>, text: String) -> Function{
        Function {
            name: name,
            body: body,
            text: text,
            pid: None,
            fds: FileDescs::new(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Function> {
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
 
         if let Some(c) = abst_command::parse(text, conf){
             Some( Function::new(name, c, ans_text) )
         }else{
             text.rewind(backup);
             None
         }
    }
}
