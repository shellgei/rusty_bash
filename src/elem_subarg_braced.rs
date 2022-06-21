//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::abst_command_elem::CommandElem;
use crate::ShellCore;
use crate::Feeder;
use crate::elem_arg::Arg;
use crate::scanner::*;

use crate::elem_arg::arg_in_brace;
use crate::abst_arg_elem::ArgElem;
use crate::utils::combine_with;

pub struct SubArgBraced {
    pub text: String,
    pub pos: DebugInfo,
    pub args: Vec<Arg>,
    pub complete: bool,
}

impl ArgElem for SubArgBraced {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<Vec<String>> {
        if self.complete {
            self.eval_complete(conf)
        }else{
            self.eval_incomplete(conf)
        }
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgBraced {
    fn new(text: &mut Feeder) -> SubArgBraced{
        SubArgBraced {
            text: "".to_string(),
            pos: DebugInfo::init(text),
            args: vec!(),
            complete: false,
        }
    }

    fn eval_complete(&mut self, conf: &mut ShellCore) -> Vec<Vec<String>> {
        let mut ans = vec!();
        for arg in &mut self.args {
            ans.push(arg.eval(conf));
        };
        ans
    }

    fn eval_incomplete(&mut self, conf: &mut ShellCore) -> Vec<Vec<String>> {
        if self.args.len() == 0 {
            return vec!(vec!(self.text.clone()));
        }else if self.args.len() == 1 {
            let mut ans = vec!();
            let mut v = "{".to_string();
            v += &self.args[0].eval(conf).join(" ");
            if let Some(c) = self.text.chars().last() {
                if c == ',' || c == '}' {
                    ans.push(v + &c.to_string());
                }else{
                    ans.push(v);
                }
            }
            return vec!(ans);
        }

        let mut ans = vec!();
        for arg in &mut self.args {
            let vs = arg.eval(conf);
            ans = combine_with(&ans, &vs, ",");
        };

        for v in &mut ans {
            *v = "{".to_owned() + v;
            if let Some(c) = self.text.chars().last() {
                if c == ',' || c == '}' {
                    *v += &c.to_string();
                }
            };
        }

        vec!(ans)
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SubArgBraced> {
        if text.len() == 0 {
            return None;
        }

        let pos = scanner_until(text, 0, "{");
        if pos != 0 {
            return None;
        }

        let mut ans = SubArgBraced::new(text);
        ans.text = text.consume(1);

        while let Some(arg) = arg_in_brace(text, conf) {
            ans.text += &arg.text.clone();
            ans.args.push(arg); 

            if text.len() == 0 
                || scanner_end_of_com(text, 0) == 1
                || text.nth(0) == ' ' {
                return Some(ans);
            };
    
            if scanner_until(text, 0, ",") == 0 {
                ans.text += &text.consume(1);
                continue;
            }else if scanner_until(text, 0, "}") == 0 {
                ans.complete = true;
                ans.text += &text.consume(1);
                break;
            };
        };

        if ans.args.len() < 2 {
            ans.complete = false;
            return Some(ans);
        }
    
        Some(ans)
    }
}
