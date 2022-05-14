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
    fn parse_info(&self) -> String {
        "".to_string()
    }
}

/* delimiter */
#[derive(Debug)]
pub struct Delim {
    pub text: String,
    //pub text_pos: usize
    pub pos: TextPos,
}

impl BashElem for Delim {
    fn parse_info(&self) -> String {
        format!("    delimiter: '{}' ({})\n", self.text.clone(), self.pos.text())
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
    fn parse_info(&self) -> String {
        format!("    end mark : '{}' ({})\n", self.text.clone(), self.pos.text())

    }
}

#[derive(Debug)]
pub struct Arg {
    pub text: String,
    pub pos: TextPos,
    pub subargs: Vec<SubArg>
}

/* arg, subarg */
#[derive(Debug)]
pub struct SubArg {
    pub text: String,
    pub pos: TextPos,
    pub quote: Option<char>,
}

impl BashElem for Arg {
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }

    fn eval(&self) -> Option<String> {
        let mut ans = "".to_string();
        for sub in &self.subargs {
            if let Some(s) = sub.eval(){
                ans += &s;
            };
        }

        Some(ans)
    }
}

impl BashElem for SubArg {
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }

    fn eval(&self) -> Option<String> {
        match self.quote {
            Some('\'') => Some(self.text[1..self.text.len()-1].to_string().clone()),
            Some('"')  => Some(SubArg::remove_escape(&self.text[1..self.text.len()-1].to_string().clone())),
            _          => Some(SubArg::remove_escape(&self.text.clone())),
        }
    }
}

impl SubArg {
    fn remove_escape(text: &String) -> String{
        let mut escaped = false;
        let mut ans = "".to_string();
        
        for ch in text.chars() {
            if escaped {
                ans.push(ch);
                escaped = false;
            }else{ //not secaped
                if ch == '\\' {
                    escaped = true;
                }else{
                    ans.push(ch);
                    escaped = false;
                };
            };
        }
        ans
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
        let mut args = Vec::<String>::new();

        for elem in &self.elems {
            if let Some(arg) = &elem.eval() {
                args.push(arg.clone());
            }
        };

        args
    }

    fn exec_external_command(&self, args: &Vec<String>, conf: &mut ShellCore) {
        let cargs: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        if conf.flags.d {
            eprintln!("{}", self.parse_info());
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
