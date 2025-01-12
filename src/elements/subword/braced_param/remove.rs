//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::BracedParam;
use crate::elements::subword::braced_param::Word;
use crate::utils::glob;

#[derive(Debug, Clone, Default)]
pub struct Remove {
    pub remove_symbol: String,
    pub remove_pattern: Option<Word>,
}

pub fn set(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let pattern = match &obj.remove.as_mut().unwrap().remove_pattern {
        None => return true,
        Some(w) => {
            match w.eval_for_case_word(core) {
                Some(s) => s,
                None    => return false,
            }
        },
    };
 
    let extglob = core.shopts.query("extglob");
 
    if obj.remove.as_mut().unwrap().remove_symbol.starts_with("##") {
        let pat = glob::parse(&pattern, extglob);
        let len = glob::longest_match_length(&obj.text, &pat);
        obj.text = obj.text[len..].to_string();
    } else if obj.remove.as_mut().unwrap().remove_symbol.starts_with("#") {
        let pat = glob::parse(&pattern, extglob);
        let len = glob::shortest_match_length(&obj.text, &pat);
        obj.text = obj.text[len..].to_string();
    }else if obj.remove.as_mut().unwrap().remove_symbol.starts_with("%") {
        percent(obj, &pattern, extglob);
    }else {
        return false;
    }
    true
}

pub fn percent(obj: &mut BracedParam, pattern: &String, extglob: bool) {
    let mut length = obj.text.len();
    let mut ans_length = length;
 
    for ch in obj.text.chars().rev() {
        length -= ch.len_utf8();
        let s = obj.text[length..].to_string();
 
        if glob::parse_and_compare(&s, &pattern, extglob) {
            ans_length = length;
            if obj.remove.as_mut().unwrap().remove_symbol == "%" {
                break;
            }
        }
    }
 
    obj.text = obj.text[0..ans_length].to_string();
}
