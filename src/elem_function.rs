//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid, fork, ForkResult};
use std::os::unix::prelude::RawFd;
use crate::elem_redirect::Redirect;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::elem_compound_brace::CompoundBrace;
use crate::elem_varname::VarName;
use crate::scanner::scanner_varname;
use crate::utils_io::*;

/* ( script ) */
pub struct Function {
    pub name: String,
    pub body: CompoundBrace,
    pub redirects: Vec<Box<Redirect>>,
    pub text: String,
    pid: Option<Pid>, 
    pub pipein: RawFd,
    pub pipeout: RawFd,
    pub prevpipein: RawFd,
}

impl ScriptElem for Function {
    fn exec(&mut self, conf: &mut ShellCore) {
        if self.pipeout == -1 && self.pipein == -1 && self.prevpipein == -1 && self.redirects.len() == 0 {
             self.body.exec(conf);
             return;
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    set_child_io(self.pipein, self.pipeout, self.prevpipein, &self.redirects);
                    self.body.exec(conf);
                },
                Ok(ForkResult::Parent { child } ) => {
                    self.pid = Some(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.pipein = pin;
        self.pipeout = pout;
        self.prevpipein = pprev;
    }

    fn get_pipe_end(&mut self) -> RawFd { self.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.pipeout }
}

impl Function {
    pub fn new(name: String, body: CompoundBrace) -> Function{
        Function {
            name: name,
            body: body,
            pid: None,
            redirects: vec!(),
            text: "".to_string(),
            pipein: -1,
            pipeout: -1,
            prevpipein: -1,
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Function> {
         let var_pos = scanner_varname(text, 0);
         if var_pos == 0 {
             return None;
         }

         let backup = text.clone();
         let name = VarName::new(text, var_pos);

        let _ = ArgDelimiter::parse(text);

        if text.len() == 0 || text.nth(0) != '(' {
            text.rewind(backup);
            return None;
        }
        text.consume(1);

        if text.len() == 0 || text.nth(0) != ')' {
            text.rewind(backup);
            return None;
        }
        text.consume(1);

        let _ = ArgDelimiter::parse(text);

        if let Some(c) = CompoundBrace::parse(text, conf){
            Some( Function::new(name.text, c) )
        }else{
            text.rewind(backup);
            None
        }
    }
}
