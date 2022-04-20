//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
//
use nix::unistd::{execvp, fork, ForkResult, Pid}; 
use nix::sys::wait::*;

use std::ffi::CString;

pub struct Core {
    elems: Vec<Box<dyn Element>>,
    text: String,
    text_pos: u32
}

impl Core {
    fn info(&self){
        println!("({}[byte] text)", self.text_pos);
        println!("{}", self.text);
    }

    pub fn new() -> Core{
        Core{
            elems: Vec::new(),
            text: "".to_string(),
            text_pos: 0
        }
    }
}

trait Element {
    fn info(&self);
    fn exec(&self);
}

pub struct CommandWithArgs {
    pub core: Core,
    pub args: Box<[CString]>
}


impl CommandWithArgs {
    fn exec_command(&self) {
        execvp(&self.args[0], &*self.args).expect("Cannot exec");
    }
  
    fn wait_ext_command(child: Pid) {
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
    /*
}

impl Element for CommandWithArgs {
*/
    fn info(&self){
        self.core.info();
    }

    pub fn exec(&self){
        unsafe {
          match fork() {
              Ok(ForkResult::Child) => self.exec_command(),
              Ok(ForkResult::Parent { child } ) => CommandWithArgs::wait_ext_command(child),
              Err(err) => panic!("Failed to fork. {}", err),
          }
        }
    }
}


