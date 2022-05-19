//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::evaluator::TextPos;
use crate::BashElem;
use crate::utils::eval_glob;

pub struct Arg {
    pub text: String,
    pub pos: TextPos,
    pub subargs: Vec<Box<dyn ArgElem>>
}

impl Arg {
    fn combine(left: &Vec<String>, right: &Vec<String>) -> Vec<String> {
        if left.len() == 0 {
            return right.clone();
        };

        let mut ans = vec!();
        for lstr in left {
            let mut con = right
                .iter()
                .map(|r| lstr.clone() + &r.clone())
                .collect();

            ans.append(&mut con);
        }
        ans
    }

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

impl BashElem for Arg {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("    arg      : '{}' ({})", self.text.clone(), self.pos.text()));
        for sub in &self.subargs {
            ans.push("        subarg      : ".to_owned() + &*sub.get_text());
        };

        ans
    }

    fn eval(&self) -> Vec<String> {
        let subevals = self.subargs
            .iter()
            .map(|sub| sub.eval())
            .collect::<Vec<Vec<String>>>();

        if subevals.len() == 0 {
            return vec!();
        };

        let mut strings = vec!();
        for ss in subevals {
            strings = Arg::combine(&strings, &ss);
        }
        strings
    }
}

pub trait ArgElem {
    fn eval(&self) -> Vec<String> {
        vec!()
    }

    fn get_text(&self) -> String;
    fn get_length(&self) -> usize;
}

pub struct SubArg {
    pub text: String,
    pub pos: TextPos,
}

impl ArgElem for SubArg {
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn get_length(&self) -> usize {
        self.pos.length
    }

    fn eval(&self) -> Vec<String> {
        vec!(self.text.clone())
    }
}


pub struct SubArgDoubleQuoted {
    pub text: String,
    pub pos: TextPos,
}

impl ArgElem for SubArgDoubleQuoted {
    fn eval(&self) -> Vec<String> {
        let strip = self.text[1..self.text.len()-1].to_string();
        let s = strip.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn get_length(&self) -> usize {
        self.pos.length
    }
}

pub struct SubArgSingleQuoted {
    pub text: String,
    pub pos: TextPos,
}

impl ArgElem for SubArgSingleQuoted {
    fn eval(&self) -> Vec<String> {
        let strip = self.text[1..self.text.len()-1].to_string();
        let s = strip.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(s)
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn get_length(&self) -> usize {
        self.pos.length
    }
}

pub struct SubArgBraced {
    pub text: String,
    pub pos: TextPos,
    pub args: Vec<Arg>
}

impl ArgElem for SubArgBraced {
    fn eval(&self) -> Vec<String> {
        if self.args.len() == 0{
            return vec!("{}".to_string());
        }else if self.args.len() == 1{
            return vec!("{".to_owned() + &self.args[0].text.clone() + "}");
        };

        let mut ans = vec!();
        for arg in &self.args {
            ans.append(&mut arg.eval());
        };
        ans
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn get_length(&self) -> usize {
        self.pos.length
    }
}
