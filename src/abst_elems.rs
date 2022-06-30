//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;

use crate::{Feeder, ShellCore}; 

use crate::elem_compound_if::CompoundIf;
use crate::elem_compound_while::CompoundWhile;
use crate::elem_compound_paren::CompoundParen;
use crate::elem_compound_brace::CompoundBrace;

use crate::elem_subarg_command_substitution::SubArgCommandSubstitution;
use crate::elem_subarg_non_quoted::SubArgNonQuoted;
use crate::elem_subarg_double_quoted::SubArgDoubleQuoted;
use crate::elem_subarg_single_quoted::SubArgSingleQuoted;
use crate::elem_subarg_braced::SubArgBraced;
use crate::elem_subarg_variable::SubArgVariable;

pub trait ListElem {
    fn exec(&mut self, _conf: &mut ShellCore);
    fn get_text(&self) -> String;
}

pub trait PipelineElem {
    fn exec(&mut self, conf: &mut ShellCore);
    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd);
    fn get_pid(&self) -> Option<Pid>;
    fn get_pipe_end(&mut self) -> RawFd;
    fn get_pipe_out(&mut self) -> RawFd;
    fn get_eoc_string(&mut self) -> String;
    fn get_text(&self) -> String;
}

pub trait CommandElem {
    fn parse_info(&self) -> Vec<String>;
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String>;
    fn get_text(&self) -> String;
}

pub trait ArgElem {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

pub fn compound(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn PipelineElem>> {
    if let Some(a) =      CompoundIf::parse(text,conf)            {Some(Box::new(a))}
    else if let Some(a) = CompoundWhile::parse(text, conf)        {Some(Box::new(a))}
    else if let Some(a) = CompoundParen::parse(text, conf, false) {Some(Box::new(a))}
    else if let Some(a) = CompoundBrace::parse(text, conf)        {Some(Box::new(a))}
    else {None}
}

pub fn subarg(text: &mut Feeder, conf: &mut ShellCore, is_value: bool, is_in_brace: bool) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgCommandSubstitution::parse(text, conf, is_value) {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)                       {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text, conf, is_value)         {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf, is_value)   {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf, is_value)   {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse(text, is_in_brace)         {Some(Box::new(a))}
    else {None}
}
