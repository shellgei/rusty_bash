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
                None => match w.subwords.len() {
                    0 => "".to_string(),
                    _ => return false,
                },
            }
        },
    };

    let extglob = core.shopts.query("extglob");

    let mut start = 0;
    let mut ans = String::new();
    let mut skip = 0;
    for ch in obj.text.chars() {
        if skip > 0 {
            skip -= 1;
            start += ch.len_utf8();
            continue;
        }

        let len = glob::longest_match_length(&obj.text[start..].to_string(), &pattern, extglob);
        if len != 0 {
            if ! obj.all_replace {
                obj.text = [&obj.text[..start], &string_to[0..], &obj.text[start+len..] ].concat();
                return true;
            }

            skip = obj.text[start..start+len].chars().count() - 1;
            ans += &string_to.clone();
        }else{
            ans += &ch.to_string();
        }
        start += ch.len_utf8();
    }

    obj.text = ans;
    true
}
