//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

use crate::ShellCore;
use crate::abst_elems::CommandElem;
use crate::utils::blue_strings;
use crate::abst_elems::ListElem;

use crate::Feeder;
use crate::elem_substitution::Substitution;
use crate::elem_end_of_pipeline::Eop;
use crate::elem_redirect::Redirect;
use crate::scanner::*;

pub struct SetVariables {
    pub elems: Vec<Box<dyn CommandElem>>,
    pub text: String,
    pub eop: Option<Eop>,
}


impl ListElem for SetVariables {
    fn exec(&mut self, conf: &mut ShellCore) {
        //if conf.flags.d {
        if conf.has_flag('d') {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for e in &mut self.elems {
            let sub = e.eval(conf);
            let (key, value) = (sub[0].clone(), sub[1].clone());
            if let Ok(_) = env::var(&key) {
                env::set_var(key, value);
            }else{
                conf.vars.insert(key, value);
            };
        };
    }

    fn get_text(&self) -> String { self.text.clone() }

    fn get_end(&self) -> String {
        let text = if let Some(e) = &self.eop {
            e.text.clone()
        }else{
            return "".to_string();
        };

        if text.chars().count() > 1 { 
            if text.chars().nth(0) == Some('|') && text.chars().nth(1) == Some('|') {
                return "||".to_string();
            }
            if text.chars().nth(0) == Some('&') && text.chars().nth(1) == Some('&') {
                return "&&".to_string();
            }
        }
        "".to_string()
    } 
}

impl SetVariables {
    pub fn new() -> SetVariables{
        SetVariables {
            elems: vec!(),
            text: "".to_string(),
            eop: None,
        }
    }

    pub fn return_if_valid(ans: SetVariables) -> Option<SetVariables> {
        if ans.elems.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }

    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("substitutions: '{}'", self.text));
        for elem in &self.elems {
            ans.append(&mut elem.parse_info());
        };
        
        blue_strings(&ans)
    }

    pub fn push(&mut self, s: Box<dyn CommandElem>){
        self.text += &s.get_text();
        self.elems.push(s);
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SetVariables> {
        let backup = text.clone();
        let mut ans = SetVariables::new();
    
        loop {
            if let Some(result) = Substitution::parse(text, conf) {
                ans.push(Box::new(result));
            }else if let Some(r) = Redirect::parse(text){
                ans.text += &r.text;
            }else{
                break;
            }
    
            let d = scanner_while(text, 0, " \t");
            ans.text += &text.consume(d);
        }

        if scanner_end_paren(text, 0) == 1 {
        }else if let Some(result) = Eop::parse(text){
            ans.text += &result.text;
            ans.eop = Some(result);
        }else{
            text.rewind(backup);
            return None;
        }
    
        SetVariables::return_if_valid(ans)
    }
}
