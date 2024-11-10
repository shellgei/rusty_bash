//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::BracedParam;
use crate::utils::glob;

pub fn get(obj: &BracedParam, core: &mut ShellCore) -> Option<String> {
    let pattern = match &obj.remove_pattern {
        None => return Some(obj.text.clone()),
        Some(w) => {
            match w.eval_for_case_word(core) {
                Some(s) => s,
                None    => return None,
            }
        },
    };
 
    let extglob = core.shopts.query("extglob");
 
    if obj.remove_symbol.starts_with("#") {
        hash(obj, &pattern, extglob)
    }else if obj.remove_symbol.starts_with("%") {
        percent(obj, &pattern, extglob)
    }else {
        Some(obj.text.clone())
    }
}

pub fn hash(obj: &BracedParam, pattern: &String, extglob: bool) -> Option<String> {
    let mut length = 0;
    let mut ans_length = 0;
 
    for ch in obj.text.chars() {
        length += ch.len_utf8();
        let s = obj.text[0..length].to_string();
 
        if glob::compare(&s, &pattern, extglob) {
            ans_length = length;
            if obj.remove_symbol == "#" {
                break;
            }
        }
    }
 
    Some( obj.text[ans_length..].to_string() )
}

pub fn percent(obj: &BracedParam, pattern: &String, extglob: bool) -> Option<String> {
    let mut length = obj.text.len();
    let mut ans_length = length;
 
    for ch in obj.text.chars().rev() {
        length -= ch.len_utf8();
        let s = obj.text[length..].to_string();
 
        if glob::compare(&s, &pattern, extglob) {
            ans_length = length;
            if obj.remove_symbol == "%" {
                break;
            }
        }
    }
 
    Some( obj.text[0..ans_length].to_string() )
}
