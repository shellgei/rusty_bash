//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::{eval_glob, combine};
use crate::debuginfo::DebugInfo;
use crate::elems_in_arg::{VarName, ArgElem};

pub trait CommandPart {
    fn parse_info(&self) -> Vec<String>;
    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> { vec!() }
    fn text(&self) -> String { String::new() }
}

/* delimiter */
#[derive(Debug)]
pub struct ArgDelimiter {
    pub text: String,
    pub debug: DebugInfo,
}

impl CommandPart for ArgDelimiter {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    delimiter: '{}' ({})", self.text.clone(), self.debug.text()))
    }
}

/* ;, \n, and comment */
#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    pub debug: DebugInfo,
}

impl CommandPart for Eoc {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    end mark : '{}' ({})\n", self.text.clone(), self.debug.text()))
    }
}

pub struct Substitution {
    pub text: String,
    pub var: VarName,
    pub value: Arg,
    pub debug: DebugInfo,
}

impl CommandPart for Substitution {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    substitution: '{}' ({})\n", self.text.clone(), self.debug.text()))
    }

    fn eval(&self, conf: &mut ShellCore) -> Vec<String> { 
        let mut ans = vec!();
        ans.push(self.var.text.clone());
        
        let mut v = "".to_string();
        for s in self.value.eval(conf){
            v += &s;
        }
        ans.push(v);

        ans
    }
}

pub struct Arg {
    pub text: String,
    pub pos: DebugInfo,
    pub subargs: Vec<Box<dyn ArgElem>>
}

impl Arg {
    pub fn expand_glob(text: &String) -> Vec<String> {
        let mut ans = eval_glob(text);

        if ans.len() == 0 {
            let s = text.clone().replace("\\*", "*").replace("\\\\", "\\");
            ans.push(s);
        };
        ans
    }

    pub fn remove_escape(text: &String) -> String{
        let mut escaped = false;
        let mut ans = "".to_string();
        
        for ch in text.chars() {
            if escaped || ch != '\\' {
                ans.push(ch);
            };
            escaped = !escaped && ch == '\\';
        }
        ans
    }
}

impl CommandPart for Arg {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("    arg      : '{}' ({})",
                              self.text.clone(), self.pos.text()));
        for sub in &self.subargs {
            ans.push("        subarg      : ".to_owned() + &*sub.text());
        };

        ans
    }

    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        let subevals = self.subargs
            .iter()
            .map(|sub| sub.eval(conf))
            .collect::<Vec<Vec<String>>>();

        if subevals.len() == 0 {
            return vec!();
        };

        let mut strings = vec!();
        for ss in subevals {
            strings = combine(&strings, &ss);
        }
        strings
    }
}
