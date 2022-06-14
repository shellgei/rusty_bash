//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid, fork, ForkResult, pipe};
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use std::process::exit;
use crate::utils::dup_and_close;

/* ( script ) */
pub struct CompoundParen {
    pub script: Option<Script>,
    pub text: String,
    pid: Option<Pid>, 
    pub infd_expansion: RawFd,
    pub outfd_expansion: RawFd,
    pub expansion: bool,
    pub expansion_str: String,
}

impl ScriptElem for CompoundParen {
    fn exec(&mut self, conf: &mut ShellCore) -> Option<Pid>{
        if self.expansion {
            self.set_command_expansion_pipe();
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    //self.set_child_io();
                    if self.expansion {
                        dup_and_close(self.outfd_expansion, 1);
                    }
                    if let Some(s) = &mut self.script {
                        s.exec(conf);
                        exit(conf.vars["?"].parse::<i32>().unwrap());
                    };
                },
                Ok(ForkResult::Parent { child } ) => {
                    self.pid = Some(child);
                    return Some(child);
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }

        None
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_expansion(&mut self, pin: RawFd, pout: RawFd) {
        self.infd_expansion = pin;
        self.outfd_expansion = pout;
//        self.expansion = true;
    }
}

impl CompoundParen {
    pub fn new() -> CompoundParen{
        CompoundParen {
            script: None,
            pid: None,
            text: "".to_string(),
            infd_expansion: -1,
            outfd_expansion: -1,
            expansion: false,
            expansion_str: "".to_string(),
        }
    }

    fn set_command_expansion_pipe(&mut self){
        let p = pipe().expect("Pipe cannot open");
        self.set_expansion(p.0, p.1);
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundParen> {
        if text.len() == 0 || text.nth(0) != '(' {
            return None;
        }

        let backup = text.clone();
        text.consume(1);
        let mut ans = CompoundParen::new();

        if let Some(s) = Script::parse(text, conf, true) {
            ans.text = "(".to_owned() + &s.text + ")";
            ans.script = Some(s);
        }

        if text.len() == 0 || text.nth(0) != ')' {
            text.rewind(backup);
            return None;
        }

        text.consume(1);
        Some(ans)
    }

    /*
    fn wait(&self, child: Pid, conf: &mut ShellCore) -> String {
        let ans = "".to_string();

        match waitpid(child, None).expect("Faild to wait child process.") {
            WaitStatus::Exited(pid, status) => {
                conf.vars.insert("?".to_string(), status.to_string());
                if status != 0 {
                    eprintln!("Pid: {:?}, Exit with {:?}", pid, status);
                }
            }
            WaitStatus::Signaled(pid, signal, _) => {
                conf.vars.insert("?".to_string(), (128+signal as i32).to_string());
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal)
            }
            _ => {
                eprintln!("Unknown error")
            }
        };

        ans
    }
*/
}
