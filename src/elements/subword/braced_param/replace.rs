//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::BracedParam;
use crate::utils::glob;

pub fn set(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let pattern = match &obj.replace_from {
        None => return true,
        Some(w) => {
            match w.eval_for_case_word(core) {
                Some(s) => s,
                None    => return false,
            }
        },
    };

    let string_to = match &obj.replace_to {
        None => "".to_string(),
        Some(w) => {
            match w.eval_for_case_word(core) {
                Some(s) => s,
                None => return false,
            }
        },
    };

    let extglob = core.shopts.query("extglob");

    let mut start = 0;
    for ch in obj.text.chars() {
        let len = glob::longest_match_length(&obj.text[start..].to_string(), &pattern, extglob);
        if len != 0 {
            obj.text = obj.text[..start].to_string() + &string_to + &obj.text[start+len..].to_string();
            return true;
        }
        start += ch.len_utf8();
    }
    true
}

/*
pub fn replace(text: &str, pattern: &String,
               string_to: &String, extglob: bool) -> Option<String> {
    let mut length = 0;
    let mut ans_length = 0;
 
    for ch in text.chars() {
        length += ch.len_utf8();
        let s = text[0..length].to_string();
 
        if glob::compare(&s, &pattern, extglob) {
            dbg!("{:?}", glob::longest_match_length(&s, &pattern, extglob));
            ans_length = length;
        }
    }

    match ans_length != 0 {
        true => Some(string_to.to_owned() + &text[ans_length..].to_string()),
        false => None,
    }
}*/
