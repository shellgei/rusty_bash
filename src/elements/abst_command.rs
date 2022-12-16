//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;

use crate::{Feeder, ShellCore}; 

use crate::elements::compound_double_paren::CommandDoubleParen;
use crate::elements::compound_if::CommandIf;
use crate::elements::compound_while::CommandWhile;
use crate::elements::compound_paren::CommandParen;
use crate::elements::compound_brace::CommandBrace;
use crate::elements::compound_case::CommandCase;
use crate::elements::simple_command::SimpleCommand;

use std::process::exit;
use nix::unistd::{close, fork, ForkResult};

pub trait AbstCommand {
    fn exec(&mut self, conf: &mut ShellCore) {
        if self.no_connection() {
             self.exec_elems(conf);
             return;
        };

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    if let Err(s) = self.set_child_io(conf){
                        eprintln!("{}", s);
                        exit(1);
                    }
                    self.exec_elems(conf);
                    close(1).expect("Can't close a pipe end");
                    exit(conf.vars["?"].parse::<i32>().unwrap());
                },
                Ok(ForkResult::Parent { child } ) => {
                    self.set_pid(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd);
    fn get_pid(&self) -> Option<Pid>;
    fn get_pipe_end(&mut self) -> RawFd;
    fn get_pipe_out(&mut self) -> RawFd;
    fn get_text(&self) -> String;
    fn set_child_io(&mut self, _conf: &mut ShellCore) -> Result<(), String> {Ok(())}
    fn exec_elems(&mut self, _conf: &mut ShellCore) {}
    fn no_connection(&self) -> bool { true }
    fn set_pid(&mut self, _pid: Pid) {}
}

pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn AbstCommand>> {
    if let Some(a) =      CommandIf::parse(text,conf)                  {Some(Box::new(a))}
    else if let Some(a) = CommandWhile::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = CommandCase::parse(text, conf)               {Some(Box::new(a))}
    else if let Some(a) = CommandParen::parse(text, conf, false)       {Some(Box::new(a))}
    else if let Some(a) = CommandDoubleParen::parse(text, conf, false) {Some(Box::new(a))}
    else if let Some(a) = CommandBrace::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SimpleCommand::parse(text, conf)                    {Some(Box::new(a))}
    else {None}
}
