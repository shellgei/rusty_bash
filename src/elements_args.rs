//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::TextPos;
use crate::BashElem;

#[derive(Debug)]
pub struct Arg {
    pub text: String,
    pub pos: TextPos,
    pub subargs: Vec<SubArg>
}

impl Arg {
    fn combine(left: &Vec<String>, right: &Vec<String>) -> Vec<String> {
        let mut ans = vec!();

        if left.len() == 0 {
            for rstr in right {
                ans.push(rstr.clone());
            }
            return ans;
        };

        for lstr in left {
            for rstr in right {
                ans.push(lstr.clone() + &rstr.clone());
            }
        }
        ans
    }
}

#[derive(Debug)]
pub struct SubArg {
    pub text: String,
    pub pos: TextPos,
    pub quote: Option<char>,
    pub braced: bool,
}

impl BashElem for Arg {
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }

    fn eval(&self) -> Option<String> {
        let subevals = self.subargs
            .iter()
            .map(|sub| sub.eval())
            .collect::<Vec<Vec<String>>>();

        if subevals.len() == 0 {
            return None;
        };

        let mut strings = vec!();
        for ss in subevals {
            //eprintln!("subeval: {:?}", ss);
            strings = Arg::combine(&strings, &ss);
        }
        Some(strings.join(" "))
    }
}

//impl BashElem for SubArg {
impl SubArg {
    /*
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }
    */

    fn eval(&self) -> Vec<String> {
        let mut ans = vec!();
        if let Some(q) = self.quote {
            if q == '\'' {
                ans.push(self.text[1..self.text.len()-1].to_string().clone());
            }else{
                ans.push(SubArg::remove_escape(&self.text[1..self.text.len()-1].to_string().clone()));
            }
            return ans;
        };

        if self.braced {
            let mut tmp = "".to_string();
            let stripped = self.text[1..self.text.len()-1].to_string().clone();
            let mut escaped = false;
            for ch in stripped.chars() {
                if escaped {
                    escaped = false;
                    tmp.push(ch);
                }else if ch == '\\' {
                    escaped = true;
                }else if ch == ',' {
                    ans.push(tmp);
                    tmp = "".to_string();
                }else{
                    tmp.push(ch);
                };
            }
            ans.push(tmp);
            //eprintln!("expanded: {:?}", ans);
            return ans;
        };

        ans.push(SubArg::remove_escape(&self.text.clone()));
        ans
    }
}

impl SubArg {
    fn remove_escape(text: &String) -> String{
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
