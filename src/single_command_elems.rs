//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::{eval_glob, combine};
use crate::debuginfo::DebugInfo;
use crate::arg_elems::ArgElem;

pub trait SingleCommandElem {
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

impl SingleCommandElem for ArgDelimiter {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    delimiter: '{}' ({})", self.text.clone(), self.debug.text()))
    }
}

#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    pub debug: DebugInfo,
}

impl SingleCommandElem for Eoc {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    end mark : '{}' ({})\n", self.text.clone(), self.debug.text()))
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

impl SingleCommandElem for Arg {
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
