//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
//
use nix::unistd::{execvp, fork, ForkResult, Pid}; 
use nix::sys::wait::*;

use std::ffi::CString;

pub struct Tree {
    pub elems: Vec<Tree>,
    pub text: String,
    pub text_pos: u32
}

impl Tree {
    /*
    fn info(&self){
        println!("({}[byte] text)", self.text_pos);
        println!("{}", self.text);
    }
    */

    pub fn new() -> Tree{
        Tree{
            elems: Vec::new(),
            text: "".to_string(),
            text_pos: 0
        }
    }
}

/*
pub trait Element {
    fn info(&self);
    //fn eval(&self) -> Vec<CString>;
    fn exec(&self);
}
*/

/* command arg arg arg ... */
pub struct CommandWithArgs {
    pub tree: Tree,
    pub args: Box<[CString]>
}

impl CommandWithArgs {
    fn exec_command(&self) {
        execvp(&self.args[0], &*self.args).expect("Cannot exec");
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
    /*
}

impl Element for CommandWithArgs {
*/
    /*
    fn info(&self){
        self.tree.info();
    }
    */

    pub fn exec(&self){
        /*
        for e in self.tree.elems.iter() {
            e.info();
        }
        */

        unsafe {
          match fork() {
              Ok(ForkResult::Child) => self.exec_command(),
              Ok(ForkResult::Parent { child } ) => CommandWithArgs::wait_command(child),
              Err(err) => panic!("Failed to fork. {}", err),
          }
        }
    }
}


/* arg */
/*
pub struct Arg {
    pub tree: Tree,
//    pub evaluated_text: Box<String>,
}
*/
