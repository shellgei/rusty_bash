//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

use crate::{ShellCore,ElemOfCommand};
use crate::utils::blue_string;
use crate::elem_script::Executable;

use crate::Feeder;
use crate::elem_substitution::Substitution;
use crate::elem_arg::ArgDelimiter;
use crate::elem_arg::Eoc;


pub struct SetVariables {
    pub elems: Vec<Box<dyn ElemOfCommand>>,
    text: String,
}

impl SetVariables {
    pub fn new() -> SetVariables{
        SetVariables {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn return_if_valid(ans: SetVariables) -> Option<SetVariables> {
        if ans.elems.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}

impl Executable for SetVariables {
    fn exec(&mut self, conf: &mut ShellCore) -> String {
        if conf.flags.d {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for e in &mut self.elems {
            let sub = e.eval(conf);
            if sub.len() != 2{
                continue;
            };

            let (key, value) = (sub[0].clone(), sub[1].clone());
            if let Ok(_) = env::var(&key) {
                env::set_var(key, value);
            }else{
                conf.vars.insert(key, value);
            };
        };

        "".to_string()
    }
}

impl SetVariables {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("substitutions: '{}'", self.text));
        for elem in &self.elems {
            ans.append(&mut elem.parse_info());
        };
        
        blue_string(&ans)
    }

    pub fn push(&mut self, s: Box<dyn ElemOfCommand>){
        self.text += &s.text();
        self.elems.push(s);
    }

    pub fn parse(text: &mut Feeder) -> Option<SetVariables> {
        let backup = text.clone();
        let mut ans = SetVariables::new();
    
        while let Some(result) = Substitution::parse(text) {
            ans.push(Box::new(result));
    
            if let Some(result) = ArgDelimiter::parse(text){
                ans.push(Box::new(result));
            }
        }
    
        if let Some(result) = Eoc::parse(text){
            ans.push(Box::new(result));
        }else{
            text.rewind(backup);
            return None;
        }
    
        SetVariables::return_if_valid(ans)
    }
}
