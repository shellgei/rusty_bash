//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

use crate::{ShellCore,CommandPart};
use crate::utils::blue_string;
use crate::elem_command::Executable;

use crate::Feeder;
use crate::parser_args::substitution;
use crate::parser::delimiter;
use crate::parser::end_of_command;


pub struct Substitutions {
    pub elems: Vec<Box<dyn CommandPart>>,
    text: String,
}

impl Substitutions {
    pub fn new() -> Substitutions{
        Substitutions {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn return_if_valid(ans: Substitutions) -> Option<Substitutions> {
        if ans.elems.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}

impl Executable for Substitutions {
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

impl Substitutions {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("substitutions: '{}'", self.text));
        for elem in &self.elems {
            ans.append(&mut elem.parse_info());
        };
        
        blue_string(&ans)
    }

    pub fn push(&mut self, s: Box<dyn CommandPart>){
        self.text += &s.text();
        self.elems.push(s);
    }

    pub fn parse(text: &mut Feeder) -> Option<Substitutions> {
        let backup = text.clone();
        let mut ans = Substitutions::new();
    
        while let Some(result) = substitution(text) {
            ans.push(Box::new(result));
    
            if let Some(result) = delimiter(text){
                ans.push(Box::new(result));
            }
        }
    
        if let Some(result) = end_of_command(text){
            ans.push(Box::new(result));
        }else{
            text.rewind(backup);
            return None;
        }
    
        Substitutions::return_if_valid(ans)
    }
}
