//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::combine;
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::abst_elems::*;
use crate::abst_elems::ArgElem;
use crate::elements::subarg_tilde::SubArgTildePrefix;
use crate::elements::subarg_string_non_quoted::SubArgStringNonQuoted;
use crate::abst_elems::CommandElem;

pub struct Arg {
    pub text: String,
    pub pos: DebugInfo,
    pub subargs: Vec<Box<dyn ArgElem>>,
}

impl Arg {
    pub fn remove_escape(text: &String) -> String{
        let mut escaped = false;
        let mut ans = "".to_string();

        let deescape_target = |c: char| {
            "$*\" \\`{};()^<>?[]'!".chars().any(|x| x == c)
        };
        
        for ch in text.chars() {
            if escaped || ch != '\\' {
                if escaped && !deescape_target(ch) {
                    ans.push('\\');
                }
                ans.push(ch);
            };
            escaped = !escaped && ch == '\\';
        }
        ans
    }

    // single quoted arg or double quoted arg or non quoted arg 
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, is_in_brace: bool) -> Option<Arg> {
        if text.len() == 0 {
            return None;
        }

        let mut ans = Arg{
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subargs: vec![],
        };

        if let Some(result) = SubArgTildePrefix::parse(text) {
            ans.text += &result.get_text();
            ans.subargs.push(Box::new(result));
        }
    
        while let Some(result) = subarg(text, conf, is_in_brace) {
            ans.text += &(*result).get_text();
            ans.subargs.push(result);
    
            if text.len() == 0 {
                break;
            };
        };
    
        if ans.text.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}

impl CommandElem for Arg {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("    arg      : '{}' ({})",
                              self.text.clone(), self.pos.get_text()));
        for sub in &self.subargs {
            ans.push("        subarg      : ".to_owned() + &*sub.get_text());
        };

        ans
    }

    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut subevals = vec![];
        for sa in &mut self.subargs {
            let vs = sa.eval(conf);

            let mut cvs = vec![];
            if sa.permit_lf() {
                cvs = vs;
            }else{
                for v in vs {
                    let cv = v.iter().map(|s| s.replace("\n", " ")).collect();
                    cvs.push(cv);
                }
            }
            
            subevals.push(cvs);
        }

        let mut strings = vec![];

        for ss in subevals {
            strings = combine(&mut strings, ss);
        }

        let mut ans = vec![];
        for v in strings {
            ans.append(&mut v.clone());
        }
        ans
    }

    fn get_text(&self) -> String { self.text.clone() }
}

pub fn arg_in_brace(text: &mut Feeder, conf: &mut ShellCore) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subargs: vec![],
    };

    let backup = text.clone();
    if text.starts_with(",") || text.starts_with("}") {
        let tmp = SubArgStringNonQuoted {
            text: "".to_string(),
            pos: DebugInfo::init(text),
            //is_value: false,
        };
        ans.subargs.push(Box::new(tmp));
        return Some(ans);
    };

    if let Some(result) = SubArgTildePrefix::parse(text) {
        ans.text += &result.get_text();
        ans.subargs.push(Box::new(result));
    }

    while let Some(result) = subarg(text, conf, true) {
        let empty_elem = (*result).get_text().len() == 0;

        ans.text += &(*result).get_text();
        ans.subargs.push(result);

        if empty_elem {
            break;
        }
    };

    if ! text.starts_with(",") && ! text.starts_with("}") {
        text.rewind(backup);
        return None;
    }

    Some(ans)
}
