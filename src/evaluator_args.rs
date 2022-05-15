//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::evaluator::TextPos;
use crate::BashElem;
use glob::glob;

#[derive(Debug)]
pub struct Arg {
    pub text: String,
    pub pos: TextPos,
    pub subargs: Vec<SubArg>
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

    fn expand_glob(text: &String) -> Vec<String> {
        let mut ans: Vec<String> = vec!();

        if let Ok(path) = glob(&text) {
            for dir in path {
                match dir {
                    Ok(d) => {
                        if let Some(s) = d.to_str() {
                            ans.push(s.to_string());
                        };
                    },
                    _ => (),
                }
            };
        };

        if ans.len() == 0 {
            let s = text.clone().replace("\\*", "*").replace("\\\\", "\\");
            //eprintln!("deescaped: {}", s);
            ans.push(s);
        };
        //eprintln!("ANS: {:?}", ans);
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
        //eprintln!("strings: {:?}", strings);

        let mut globed_strings = vec!();
        for s in strings {
            for gs in Arg::expand_glob(&s) {
                globed_strings.push(gs);
            }
        }
        //eprintln!("globed strings: {:?}", globed_strings);
        globed_strings
    }
}

impl BashElem for SubArg {
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }

    fn eval(&self) -> Vec<String> {
        let mut ans = vec!();
        if let Some(q) = self.quote {
            let strip = self.text[1..self.text.len()-1].to_string().clone();
            let s = if q == '\'' {
                strip.replace("\\", "\\\\").replace("*", "\\*") //escape file glob
            }else{
                strip.replace("\\\"", "\"").replace("*", "\\*") //escape for file glob
            };
            ans.push(s);
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
