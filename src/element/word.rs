//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::combine;
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::element::abst_subword;
use crate::element::abst_subword::WordElem;
use crate::element::subword_tilde::SubWordTildePrefix;
use crate::element::subword_string_non_quoted::SubWordStringNonQuoted;

pub struct Word {
    pub text: String,
    pub pos: DebugInfo,
    pub subwords: Vec<Box<dyn WordElem>>,
}

impl Word {
    pub fn remove_escape(text: &String) -> String{
        let mut escaped = false;
        let mut ans = "".to_string();

        let deescape_twordet = |c: char| {
            "$*\" \\`{};()^<>?[]'!".chars().any(|x| x == c)
        };
        
        for ch in text.chars() {
            if escaped || ch != '\\' {
                if escaped && !deescape_twordet(ch) {
                    ans.push('\\');
                }
                ans.push(ch);
            };
            escaped = !escaped && ch == '\\';
        }
        ans
    }

    // single quoted word or double quoted word or non quoted word 
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, is_in_brace: bool) -> Option<Word> {
        if text.len() == 0 {
            return None;
        }

        let mut ans = Word{
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subwords: vec![],
        };

        if let Some(result) = SubWordTildePrefix::parse(text, false) {
            ans.text += &result.get_text();
            ans.subwords.push(Box::new(result));
        }
    
        while let Some(result) = abst_subword::parse_in_arg(text, conf, is_in_brace) {
            ans.text += &(*result).get_text();
            ans.subwords.push(result);
    
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

    pub fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("    word      : '{}' ({})",
                              self.text.clone(), self.pos.get_text()));
        for sub in &self.subwords {
            ans.push("        subword      : ".to_owned() + &*sub.get_text());
        };

        ans
    }

    pub fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut subevals = vec![];
        for sa in &mut self.subwords {
            let vs = sa.eval(conf, true);

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

    pub fn get_text(&self) -> String { self.text.clone() }
}

pub fn word_in_brace(text: &mut Feeder, conf: &mut ShellCore) -> Option<Word> {
    let mut ans = Word{
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subwords: vec![],
    };

    let backup = text.clone();
    if text.starts_with(",") || text.starts_with("}") {
        let tmp = SubWordStringNonQuoted {
            text: "".to_string(),
            pos: DebugInfo::init(text),
            //is_value: false,
        };
        ans.subwords.push(Box::new(tmp));
        return Some(ans);
    };

    if let Some(result) = SubWordTildePrefix::parse(text, false) {
        ans.text += &result.get_text();
        ans.subwords.push(Box::new(result));
    }

    while let Some(result) = abst_subword::parse_in_arg(text, conf, true) {
        let empty_elem = (*result).get_text().len() == 0;

        ans.text += &(*result).get_text();
        ans.subwords.push(result);

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
