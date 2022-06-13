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

pub trait ScriptElem {
    fn exec(&mut self, _conf: &mut ShellCore) -> Option<Pid> { None }
    fn set_pipe(&mut self, _pin: RawFd, _pout: RawFd, _pprev: RawFd) { }
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
