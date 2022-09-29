//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::PipelineElem;
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use crate::elem_redirect::Redirect;
use nix::unistd::Pid;
use crate::utils_io::*;
use crate::elem_end_of_command::Eoc;

/* ( script ) */
pub struct CompoundWhile {
    pub conddo: Option<(Script, Script)>,
    text: String,
    pid: Option<Pid>,
    fds: FileDescs,
    pub eoc: Option<Eoc>,
}

impl PipelineElem for CompoundWhile {
    fn get_pid(&self) -> Option<Pid> { self.pid }
    fn set_pid(&mut self, pid: Pid) { self.pid = Some(pid); }
    fn no_connection(&self) -> bool { self.fds.no_connection() }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn set_child_io(&self){
        self.fds.set_child_io();
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

    fn exec_elems(&mut self, conf: &mut ShellCore) {
        loop {
            if let Some((cond, doing)) = &mut self.conddo {
                cond.exec(conf);
                if conf.vars["?"] != "0" {
                    conf.set_var("?", "0");
                    break;
                }
                doing.exec(conf);
            }
        }
    }
}
