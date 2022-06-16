//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use nix::unistd::Pid;
use crate::elem_pipeline::Pipeline;
use crate::elem_setvars::SetVariables;
use crate::elem_blankpart::BlankPart;
use crate::ScriptElem;

pub struct Script {
    pub elems: Vec<Box<dyn ScriptElem>>,
    pub text: String,
}

impl ScriptElem for Script {
    fn exec(&mut self, conf: &mut ShellCore) -> Option<Pid>{
        for e in &mut self.elems {
            e.exec(conf);
        }

        for c in &self.elems {
            if let Some(p) = c.get_pid() {
                self.wait(p, conf);
            }
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
        let backup = text.clone();
    
        loop {
            if let Some(result) = BlankPart::parse(text)                {
                ans.text += &result.text;
                ans.elems.push(Box::new(result));
            }else if let Some(result) = SetVariables::parse(text, conf) {
                ans.text += &result.text;
                ans.elems.push(Box::new(result));
            }else if let Some(result) = Pipeline::parse(text, conf) {
                ans.text += &result.text;
                ans.elems.push(Box::new(result));
            }
            else {break}

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
            text.consume(text.len());
            None
        }
    }
}
