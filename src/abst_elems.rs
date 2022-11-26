//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;

use crate::{Feeder, ShellCore}; 

use crate::elements::compound_double_paren::CompoundDoubleParen;
use crate::elements::compound_if::CompoundIf;
use crate::elements::compound_while::CompoundWhile;
use crate::elements::compound_paren::CompoundParen;
use crate::elements::compound_brace::CompoundBrace;
use crate::elements::compound_case::CompoundCase;
use crate::elements::command::Command;

use crate::elements::subarg_command_substitution::SubArgCommandSubstitution;
use crate::elements::subarg_math_substitution::SubArgMathSubstitution;
use crate::elements::subarg_string_non_quoted::SubArgStringNonQuoted;
use crate::elements::subarg_double_quoted::SubArgDoubleQuoted;
use crate::elements::subarg_single_quoted::SubArgSingleQuoted;
use crate::elements::subarg_braced::SubArgBraced;
use crate::elements::subarg_variable::SubArgVariable;
use std::process::exit;
use nix::unistd::{close, fork, ForkResult};

pub trait ListElem {
    fn exec(&mut self, conf: &mut ShellCore);

    fn get_text(&self) -> String;
}

pub trait Compound {
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

pub trait CommandElem {
    fn parse_info(&self) -> Vec<String>;
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String>;
    fn get_text(&self) -> String;
}

pub trait ArgElem {
    fn eval(&mut self, _conf: &mut ShellCore, as_value: bool) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

pub fn compound(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn Compound>> {
    if let Some(a) =      CompoundIf::parse(text,conf)                  {Some(Box::new(a))}
    else if let Some(a) = CompoundWhile::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = CompoundCase::parse(text, conf)               {Some(Box::new(a))}
    else if let Some(a) = CompoundParen::parse(text, conf, false)       {Some(Box::new(a))}
    else if let Some(a) = CompoundDoubleParen::parse(text, conf, false) {Some(Box::new(a))}
    else if let Some(a) = CompoundBrace::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = Command::parse(text, conf)                    {Some(Box::new(a))}
    else {None}
}

pub fn subarg(text: &mut Feeder, conf: &mut ShellCore, is_in_brace: bool) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgMathSubstitution::parse(text, conf)                   {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandSubstitution::parse(text, conf)           {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)                            {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text, conf)                        {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubArgStringNonQuoted::parse(text, is_in_brace, false) {Some(Box::new(a))}
    else {None}
}

pub fn subvalue(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgMathSubstitution::parse(text, conf)               {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandSubstitution::parse(text, conf)       {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)                        {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubArgStringNonQuoted::parse(text, false, true)    {Some(Box::new(a))}
    else {None}
}
