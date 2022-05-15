//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::evaluator::TextPos;
use crate::BashElem;
use glob::glob;
use crate::parser_args::expand_brace;

pub struct Arg {
    pub text: String,
    pub pos: TextPos,
//    pub subargs: Vec<SubArg>
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
                globed_strings.push(SubArg::remove_escape(&gs));
            }
        }
        //eprintln!("globed strings: {:?}", globed_strings);
        globed_strings
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
//    pub quote: Option<char>,
    pub braced: bool,
}

impl ArgElem for SubArg {
    /*
    fn parse_info(&self) -> String {
        format!("    arg      : '{}' ({})\n", self.text.clone(), self.pos.text())
    }
    */
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn get_length(&self) -> usize {
        self.pos.length
    }

    fn eval(&self) -> Vec<String> {
        if self.braced {
            expand_brace(&self.text)
        }else{
            vec!(self.text.clone())
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
