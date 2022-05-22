//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::evaluator::DebugInfo;
use crate::BashElem;
use crate::utils::{combine,eval_glob};
use crate::ShellCore;

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

impl BashElem for Arg {
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

pub trait ArgElem {
    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> {
        vec!()
    }

    fn text(&self) -> String;
//    fn get_length(&self) -> usize;
}

pub struct SubArg {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArg {
    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }
    */

    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> {
        vec!(self.text.clone())
    }
}


pub struct SubArgDoubleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    pub subargs: Vec<Box<dyn ArgElem>>
}

impl ArgElem for SubArgDoubleQuoted {
    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        let mut text = "".to_string();
        for a in &self.subargs {
            let sub = a.eval(conf);
            text += &sub[0];
        };

        let s = text.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }
    */
}

pub struct SubArgSingleQuoted {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgSingleQuoted {
    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> {
        let strip = self.text[1..self.text.len()-1].to_string();
        let s = strip.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }
    */
}

pub struct SubArgBraced {
    pub text: String,
    pub pos: DebugInfo,
    pub args: Vec<Arg>
}

impl ArgElem for SubArgBraced {
    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        if self.args.len() == 0{
            return vec!("{}".to_string());
        }else if self.args.len() == 1{
            return vec!("{".to_owned() + &self.args[0].text.clone() + "}");
        };

        let mut ans = vec!();
        for arg in &self.args {
            ans.append(&mut arg.eval(conf));
        };
        ans
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }*/
}

pub struct SubArgVariable {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgVariable {
    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        let name = if self.text.rfind('}') == Some(self.text.len()-1) {
            self.text[2..self.text.len()-1].to_string()
        }else{
            self.text[1..].to_string()
        };
        vec!(conf.get_var(&name))
    }

    fn text(&self) -> String {
        self.text.clone()
    }

    /*
    fn get_length(&self) -> usize {
        self.text.len()
    }*/
}
