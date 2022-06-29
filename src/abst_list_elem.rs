//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore};
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::Feeder; 
use crate::elem_compound_if::CompoundIf;
use crate::elem_compound_paren::CompoundParen;
use crate::elem_compound_brace::CompoundBrace;

pub trait ListElem {
    fn exec(&mut self, _conf: &mut ShellCore) { }
    fn set_pipe(&mut self, _pin: RawFd, _pout: RawFd, _pprev: RawFd) { }
    fn get_pid(&self) -> Option<Pid> { None }
    fn set_parent_io(&mut self) { }
    fn get_pipe_end(&mut self) -> RawFd { -1 }
    fn get_pipe_out(&mut self) -> RawFd { -1 }
    fn get_eoc_string(&mut self) -> String { "".to_string() }
    fn get_text(&self) -> String;
}

pub trait PipelineElem {
    fn exec(&mut self, _conf: &mut ShellCore) { }
    fn set_pipe(&mut self, _pin: RawFd, _pout: RawFd, _pprev: RawFd) { }
    fn get_pid(&self) -> Option<Pid> { None }
    fn set_parent_io(&mut self) { }
    fn get_pipe_end(&mut self) -> RawFd { -1 }
    fn get_pipe_out(&mut self) -> RawFd { -1 }
    fn get_eoc_string(&mut self) -> String { "".to_string() }
    fn get_text(&self) -> String;
}

pub fn compound(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn PipelineElem>> {
    if let Some(a) =      CompoundIf::parse(text,conf)            {Some(Box::new(a))}
    else if let Some(a) = CompoundParen::parse(text, conf, false) {Some(Box::new(a))}
    else if let Some(a) = CompoundBrace::parse(text, conf)        {Some(Box::new(a))}
    else {None}
}
