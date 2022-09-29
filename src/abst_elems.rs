//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;

use crate::{Feeder, ShellCore}; 


use crate::elem_subarg_command_substitution::SubArgCommandSubstitution;
use crate::elem_subarg_non_quoted::SubArgNonQuoted;
use crate::elem_subarg_double_quoted::SubArgDoubleQuoted;
use crate::elem_subarg_single_quoted::SubArgSingleQuoted;
use crate::elem_subarg_braced::SubArgBraced;
use crate::elem_subarg_variable::SubArgVariable;
use std::process::exit;
use nix::unistd::{close, fork, ForkResult};

pub trait ListElem {
    fn exec(&mut self, conf: &mut ShellCore);

    fn get_text(&self) -> String;
    fn get_end(&self) -> String;
}

pub trait PipelineElem {
    fn exec(&mut self, conf: &mut ShellCore) {
    }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd);
    fn get_pid(&self) -> Option<Pid>;
    fn get_pipe_end(&mut self) -> RawFd;
    fn get_pipe_out(&mut self) -> RawFd;
    fn get_eoc_string(&mut self) -> String;
    fn get_text(&self) -> String;
    fn set_child_io(&self) {}
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
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

pub fn subarg(text: &mut Feeder, conf: &mut ShellCore, is_value: bool, is_in_brace: bool) -> Option<Box<dyn ArgElem>> {
    None
}
