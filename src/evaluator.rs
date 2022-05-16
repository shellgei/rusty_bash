//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{execvp, fork, ForkResult, Pid}; 
use nix::sys::wait::*;
use std::ffi::CString;
use crate::ShellCore;
use crate::evaluator_args::Arg;

pub trait BashElem {
    fn blue_string(&self, strings: &Vec<String>) -> Vec<String> {
        strings
            .iter()
            .map(|s| format!("\x1b[34m{}\x1b[m", s))
            .collect()
    }
    fn parse_info(&self) -> Vec<String>;
    fn exec(&self, _conf: &mut ShellCore){}
    fn eval(&self) -> Vec<String> {
        vec!()
    }
}

#[derive(Debug)]
pub struct TextPos {
    pub lineno: u32,
    pub pos: u32,
    pub length: usize,
}

impl TextPos {
    pub fn text(&self) -> String {
        format!("lineno: {}, pos: {}, length: {}", 
                self.lineno.to_string(),
                self.pos.to_string(),
                self.length.to_string())
    }
}

/* empty element */
pub struct Empty { }
impl BashElem for Empty {
    fn parse_info(&self) -> Vec<String> {
        vec!()
    }
}

/* delimiter */
#[derive(Debug)]
pub struct Delim {
    pub text: String,
    pub pos: TextPos,
}

impl BashElem for Delim {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    delimiter: '{}' ({})\n", self.text.clone(), self.pos.text()))
    }
}

/* end of command */
#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    //pub text_pos: usize
    pub pos: TextPos,
}

impl BashElem for Eoc {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    end mark : '{}' ({})\n", self.text.clone(), self.pos.text()))

    }
}

/* command: delim arg delim arg delim arg ... eoc */
pub struct CommandWithArgs {
    pub elems: Vec<Box<dyn BashElem>>,
    pub text: String,
    pub text_pos: usize
}

impl BashElem for CommandWithArgs {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("command: '{}'\n", self.text));
        for elem in &self.elems {
            ans.append(&mut elem.parse_info());
        };
        
        self.blue_string(&ans)
    }

    fn exec(&self, conf: &mut ShellCore){
        let args = self.eval_args();

        if let Some(func) = conf.get_internal_command(&args[0]) {
            func(&args);
            return;
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => self.exec_external_command(&args, conf),
                Ok(ForkResult::Parent { child } ) => CommandWithArgs::wait_command(child),
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }
}

impl CommandWithArgs {
    fn eval_args(&self) -> Vec<String> {
        let mut args = vec!();

        for elem in &self.elems {
            for s in &elem.eval() {
                args.append(&mut Arg::expand_glob(&s.clone()));
            }
        };

        args.iter()
            .map(|a| Arg::remove_escape(&a))
            .collect()
    }

    fn exec_external_command(&self, args: &Vec<String>, conf: &mut ShellCore) {
        let cargs: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        if conf.flags.d {
            for s in self.parse_info() {
                eprintln!("{}", s);
            };
        };
        execvp(&cargs[0], &*cargs).expect("Cannot exec");
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
