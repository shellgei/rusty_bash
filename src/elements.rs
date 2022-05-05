//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
//
//
use nix::unistd::{execvp, fork, ForkResult, Pid}; 
use nix::sys::wait::*;
use std::ffi::CString;
use crate::ShellCore;

pub trait BashElem {
    fn blue_string(&self, text: String) -> String {
        format!("\x1b[34m{}\x1b[m", text)
    }
    fn parse_info(&self) -> String;
    fn exec(&self, _conf: &mut ShellCore){}
    fn eval(&self) -> Option<String> {
        return None
    }
}

/* empty element */
pub struct Empty { }
impl BashElem for Empty {
    fn parse_info(&self) -> String {
        "".to_string()
    }
}

/* delimiter */
#[derive(Debug)]
pub struct Delim {
    pub text: String,
    pub text_pos: usize
}

impl BashElem for Delim {
    fn parse_info(&self) -> String {
        format!("    delimiter: '{}'\n", self.text.clone())
    }
}

/* end of command */
#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    pub text_pos: usize
}

impl BashElem for Eoc {
    fn parse_info(&self) -> String {
        format!("    end mark : '{}'\n", self.text.clone())
    }
}

/* arg */
#[derive(Debug)]
pub struct Arg {
    pub text: String,
    pub text_pos: usize
}

impl BashElem for Arg {
    fn parse_info(&self) -> String {
        format!("    arg      : '{}'\n", self.text.clone())
    }

    fn eval(&self) -> Option<String> {
        Some(self.text.clone())
    }
}

/* command: delim arg delim arg delim arg ... eoc */
pub struct CommandWithArgs {
    pub elems: Vec<Box<dyn BashElem>>,
    pub text: String,
    pub text_pos: usize
}

impl BashElem for CommandWithArgs {
    fn parse_info(&self) -> String {
        let mut ans = format!("command: '{}'\n", self.text);
        for elem in &self.elems {
            ans += &elem.parse_info();
        };
        
        self.blue_string(ans)
    }

    fn exec(&self, conf: &mut ShellCore){
        if self.exec_internal_command(conf) {
            return;
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => self.exec_external_command(conf),
                Ok(ForkResult::Parent { child } ) => CommandWithArgs::wait_command(child),
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }
}

impl CommandWithArgs {
    fn exec_external_command(&self, _conf: &mut ShellCore) {
        let mut args = Vec::<CString>::new();

        for elem in &self.elems {
            if let Some(arg) = &elem.eval() {
                args.push(CString::new(arg.clone()).unwrap());
            }
        };

        execvp(&args[0], &*args).expect("Cannot exec");
    }

    fn exec_internal_command(&self, conf: &mut ShellCore) -> bool {
        let mut args = Vec::<CString>::new();

        for elem in &self.elems {
            if let Some(arg) = &elem.eval() {
                args.push(CString::new(arg.clone()).unwrap());
            }
        };

        if conf.internal_commands.contains_key(&args[0]) {
            ShellCore::exec_internal_command(conf.internal_commands[&args[0]]);
            true
        }else{
            false
        }

    }
  
    fn wait_command(child: Pid) {
        match waitpid(child, None)
            .expect("Faild to wait child process.") {
            WaitStatus::Exited(pid, status) => {
                if status != 0 {
                    println!("Pid: {:?}, Exit with {:?}", pid, status);
                };
            }
            WaitStatus::Signaled(pid, signal, _) => {
                println!("Pid: {:?}, Signal: {:?}", pid, signal)
            }
            _ => {
                println!("Unknown error")
            }
        };
    }
}
