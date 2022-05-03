//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
//
use nix::unistd::{execvp, fork, ForkResult, Pid}; 
use nix::sys::wait::*;

use std::ffi::CString;

/* delimiter */
#[derive(Debug)]
pub struct Delim {
    pub text: String,
    pub text_pos: usize
}

/* end of command */
#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    pub text_pos: usize
}

/* arg */
#[derive(Debug)]
pub struct Arg {
    pub text: String,
    pub text_pos: usize
}

/* command: delim arg delim arg delim arg ... eoc */
#[derive(Debug)]
pub struct CommandWithArgs {
    pub args: Vec<Arg>,
    pub delims: Vec<Delim>,
    pub end: Vec<Eoc>,
    pub text: String,
    pub text_pos: usize
}

impl CommandWithArgs {
    pub fn print(&self) {
        eprintln!("\x1b[34m{}\x1b[m", self.text);
    }

    fn exec_command(&self) {
        let mut args = Vec::<CString>::new();
        for e in &self.args {
            args.push(CString::new(e.text.clone()).unwrap());
        }

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


