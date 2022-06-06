//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::{eval_glob, combine};
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::abst_elem_argelem::subarg;
use crate::abst_elem_argelem::subvalue;
use crate::abst_elem_argelem::subarg_in_brace;
use crate::scanner::scanner_end_of_com;
use crate::abst_elem_argelem::ArgElem;
use crate::elem_subarg_non_quoted::SubArgNonQuoted;

pub trait ElemOfCommand {
    fn parse_info(&self) -> Vec<String>;
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> { vec!() }
    fn text(&self) -> String { String::new() }
}

/* ;, \n, and comment */
#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    pub debug: DebugInfo,
}

impl ElemOfCommand for Eoc {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    end mark : '{}' ({})\n", self.text.clone(), self.debug.text()))
    }

    fn text(&self) -> String { self.text.clone() }
}

impl Eoc {
    pub fn parse(text: &mut Feeder) -> Option<Eoc> {
        if text.len() == 0 {
            return None;
        };
    
        let pos = scanner_end_of_com(text, 0);
        if pos == 0 {
            return None;
        };
    
        Some(Eoc{text: text.consume(pos), debug: DebugInfo::init(&text)})
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

    // single quoted arg or double quoted arg or non quoted arg 
    pub fn parse(text: &mut Feeder, expand_brace: bool) -> Option<Arg> {
        let mut ans = Arg{
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subargs: vec!(),
        };
    
        let sub = if expand_brace{subarg}else{subvalue};
    
        while let Some(result) = sub(text) {
            ans.text += &(*result).text();
            ans.subargs.push(result);
    
            if text.len() == 0 {
                break;
            };
        };
    
        Some(ans)
    }
}

impl ElemOfCommand for Arg {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("    arg      : '{}' ({})",
                              self.text.clone(), self.pos.text()));
        for sub in &self.subargs {
            ans.push("        subarg      : ".to_owned() + &*sub.text());
        };

        ans
    }

    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut subevals = vec!();
        for sa in &mut self.subargs {
            subevals.push(sa.eval(conf));
        }

        if subevals.len() == 0 {
            return vec!();
        };

        let mut strings = vec!();
        for ss in subevals {
            strings = combine(&strings, &ss);
        }
        strings
    }

    fn text(&self) -> String { self.text.clone() }
}

pub fn arg_in_brace(text: &mut Feeder) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subargs: vec!(),
    };

    if text.match_at(0, ",}"){ // zero length arg
        let tmp = SubArgNonQuoted{
            text: "".to_string(),
            pos: DebugInfo::init(text),
        };
        ans.subargs.push(Box::new(tmp));
        return Some(ans);
    };

    while let Some(result) = subarg_in_brace(text) {
        ans.text += &(*result).text();
        ans.subargs.push(result);
    };

    Some(ans)
}
