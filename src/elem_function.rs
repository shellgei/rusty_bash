//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid, fork, ForkResult};
use std::os::unix::prelude::RawFd;
use std::process::exit;
use crate::elem_redirect::Redirect;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::elem_compound_brace::CompoundBrace;
use crate::elem_varname::VarName;
use crate::scanner::scanner_varname;
use crate::utils_io::*;

/* ( script ) */
pub struct Function {
    pub name: Option<VarName>,
    pub body: Option<CompoundBrace>,
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
            if let Some(s) = &mut self.body {
                s.exec(conf);
                return;
            };
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    set_child_io(self.pipein, self.pipeout, self.prevpipein, &self.redirects);
                    if let Some(s) = &mut self.body {
                        s.exec(conf);
                        exit(conf.vars["?"].parse::<i32>().unwrap());
                    };
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

    /*
    fn set_parent_io(&mut self) {
        if self.pipeout >= 0 {
            close(self.pipeout).expect("Cannot close outfd");
        };
       // return self.pipein;
    }*/

    fn get_pipe_end(&mut self) -> RawFd { self.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.pipeout }
}

impl Function {
    pub fn new() -> Function{
        Function {
            name: None,
            body: None,
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
         let mut ans = Function::new();
         ans.name = Some(VarName::new(text, var_pos));

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
            ans.body = Some(c);
        }else{
            text.rewind(backup);
            return None;
        }
        Some(ans)
    }
}
