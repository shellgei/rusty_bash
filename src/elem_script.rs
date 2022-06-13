//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use nix::unistd::Pid;
use crate::elem_pipeline::Pipeline;
//use crate::elem_script::ScriptElem;
use crate::elem_setvars::SetVariables;
use crate::elem_blankpart::BlankPart;
use crate::elem_compound_paren::CompoundParen;
use std::os::unix::prelude::RawFd;
use nix::sys::wait::waitpid;
use nix::sys::wait::WaitStatus;
use nix::unistd::read;

pub trait ScriptElem {
    fn exec(&mut self, _conf: &mut ShellCore) -> Option<Pid> { None }
    fn set_pipe(&mut self, _pin: RawFd, _pout: RawFd, _pprev: RawFd) { }
    fn set_expansion(&mut self, _pin: RawFd, _pout: RawFd) { }
    fn is_expansion(&self) -> bool { false }
    fn get_pid(&self) -> Option<Pid> { None }
    fn set_parent_io(&mut self) -> RawFd { -1 }
    fn get_expansion_infd(&self) -> RawFd { -1 }

    fn wait(&self, com: &Box<dyn ScriptElem>, child: Pid, conf: &mut ShellCore) -> String {
        let mut ans = "".to_string();

        if com.is_expansion() {
            let mut ch = [0;1000];
            //while let Ok(n) = read(com.infd_expansion, &mut ch) {
            while let Ok(n) = read(com.get_expansion_infd(), &mut ch) {
                ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
                if n < 1000 {
                    break;
                };
            };
        }

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

        if let Some(c) = ans.chars().last() {
            if c == '\n' {
                return ans[0..ans.len()-1].to_string();
            }
        }
        ans
    }
}

pub struct Script {
    pub elems: Vec<Box<dyn ScriptElem>>,
    pub text: String,
}

impl ScriptElem for Script {

    fn exec(&mut self, conf: &mut ShellCore) -> Option<Pid>{
        for p in &mut self.elems {
            p.exec(conf);
        }
        None
    }
}

impl Script {
    pub fn new() -> Script{
        Script {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, next: bool) -> Option<Script> {
        if text.len() == 0 {
            return None;
        };
    
        if text.nth(0) == ')' {
            eprintln!("Unexpected symbol: )");
            return None;
        }

        let mut ans = Script::new();
    
        loop {
            if let Some(result) = CompoundParen::parse(text, conf) {ans.elems.push(Box::new(result));}
            if let Some(result) = BlankPart::parse(text)           {ans.elems.push(Box::new(result));}
            if let Some(result) = SetVariables::parse(text, conf)  {ans.elems.push(Box::new(result));}
            if let Some(result) = Pipeline::parse(text, conf)      {ans.elems.push(Box::new(result));}

            if text.len() == 0 {
                break;
            };
        
            if text.nth(0) == ')' {
                break;
            }

            if !next {
                break;
            }
        }
    
        if ans.elems.len() > 0 {
            Some(ans)
        }else{
            eprintln!("Unknown phrase");
            None
        }
    }
}
