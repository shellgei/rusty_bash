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

/* arg, subarg */
#[derive(Debug)]
pub struct SubArg {
    pub text: String,
    pub pos: TextPos,
    pub quote: Option<char>,
}

impl BashElem for Arg {
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }

    fn eval(&self) -> Option<String> {
        let v = self.subargs
            .iter()
            .map(|sub| if let Some(s) = sub.eval(){s}else{"".to_string()})
            .collect::<Vec<String>>()
            .join("");

        Some(v)
    }
}

impl BashElem for SubArg {
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }

    fn eval(&self) -> Option<String> {
        match self.quote {
            Some('\'') => Some(self.text[1..self.text.len()-1].to_string().clone()),
            Some('"')  => Some(SubArg::remove_escape(&self.text[1..self.text.len()-1].to_string().clone())),
            _          => Some(SubArg::remove_escape(&self.text.clone())),
        }
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
