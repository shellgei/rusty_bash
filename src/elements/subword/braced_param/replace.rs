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
    replace(obj, &pattern, &string_to, extglob);
    true
}

pub fn replace(obj: &mut BracedParam, pattern: &String,
               string_to: &String, extglob: bool) {
    let mut length = 0;
    let mut ans_length = 0;
 
    for ch in obj.text.chars() {
        length += ch.len_utf8();
        let s = obj.text[0..length].to_string();
 
        if glob::compare(&s, &pattern, extglob) {
            ans_length = length;
            /*
            if obj.remove_symbol == "#" {
                break;
            }*/
        }
    }

    obj.text = string_to.to_owned() + &obj.text[ans_length..].to_string();
}
