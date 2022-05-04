//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
//
//
use nix::unistd::{execvp, fork, ForkResult, Pid}; 
use nix::sys::wait::*;

use std::any::Any;
use std::ffi::CString;

pub trait BashElem {
    fn blue_string(text: String) -> String {
        format!("\x1b[34m{}\x1b[m", text)
    }
    fn parse_info(&self) -> String;
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
}

/* command: delim arg delim arg delim arg ... eoc */
#[derive(Debug)]
pub struct CommandWithArgs {
    pub elems: Vec<Box<dyn Any>>,
    pub text: String,
    pub text_pos: usize
}


impl BashElem for CommandWithArgs {
    fn parse_info(&self) -> String {
        let mut ans = format!("command: '{}'\n", self.text);
        for elem in &self.elems {
            if let Some(e) = elem.downcast_ref::<Arg>(){
                ans += &e.parse_info();
            }else if let Some(e) = elem.downcast_ref::<Delim>(){
                ans += &e.parse_info();
            }else if let Some(e) = elem.downcast_ref::<Eoc>(){
                ans += &e.parse_info();
            }
        };
        
        Self::blue_string(ans)
    }
}

impl CommandWithArgs {
    fn exec_command(&self) {
        let mut args = Vec::<CString>::new();

        for elem in &self.elems {
            if let Some(e) = elem.downcast_ref::<Arg>(){
                args.push(CString::new(e.text.clone()).unwrap());
            };
        };

        execvp(&args[0], &*args).expect("Cannot exec");
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

    pub fn exec(&self){
        unsafe {
            match fork() {
                Ok(ForkResult::Child) => self.exec_command(),
                Ok(ForkResult::Parent { child } ) => CommandWithArgs::wait_command(child),
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }
}
