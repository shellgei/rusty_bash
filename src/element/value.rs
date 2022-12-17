//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::combine;
use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::element::abst_subword;
use crate::element::abst_subword::WordElem;
use crate::element::subword_tilde::SubWordTildePrefix;

pub struct Value {
    pub text: String,
    pub pos: DebugInfo,
    pub subvalues: Vec<Box<dyn WordElem>>,
}

impl Value {
    pub fn new() -> Value {
        Value {
            text: "".to_string(),
            pos: DebugInfo{lineno: 0, pos: 0, comment: "".to_string()},
            subvalues: vec![],
        }
    }

    // single quoted word or double quoted word or non quoted word 
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Value> {
        if text.len() == 0 {
            return None;
        }

        let mut ans = Value{
            text: "".to_string(),
            pos: DebugInfo::init(text),
            subvalues: vec![],
        };

        if let Some(result) = SubWordTildePrefix::parse(text, true) {
            ans.text += &result.get_text();
            ans.subvalues.push(Box::new(result));
        }
    
        while let Some(result) = abst_subword::parse_in_value(text, conf) {
            ans.text += &(*result).get_text();
            ans.subvalues.push(result);
    
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

    pub fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut subevals = vec![];
        for sa in &mut self.subvalues {
            let vs = sa.eval(conf, false);
            subevals.push(vs);
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

}

