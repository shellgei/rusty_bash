//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::BracedParam;
use crate::elements::subword::braced_param::Word;
use crate::utils::glob;

#[derive(Debug, Clone, Default)]
pub struct Replace {
    pub head_only_replace: bool,
    pub tail_only_replace: bool,
    pub all_replace: bool,
    pub replace_from: Option<Word>,
    pub replace_to: Option<Word>,
    pub has_replace_to: bool,
}

pub fn set(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    if obj.replace.is_none() {
        return false;
    }

    let info = obj.replace.clone().unwrap();

    let pattern = match info.replace_from {
        None => return true,
        Some(w) => {
            match w.eval_for_case_word(core) {
                Some(s) => s,
                None    => return false,
            }
        },
    };

    let string_to = match &info.replace_to {
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
        if start != 0 && info.head_only_replace {
            return true;
        }
        if skip > 0 {
            skip -= 1;
            start += ch.len_utf8();
            continue;
        }

        let pat = glob::parse(&pattern, extglob);
        let len = glob::longest_match_length(&obj.text[start..].to_string(), &pat);
        if len != 0 && info.tail_only_replace {
            if len == obj.text[start..].len() {
                obj.text = [&obj.text[..start], &string_to[0..] ].concat();
                return true;
            }else{
                ans += &ch.to_string();
                start += ch.len_utf8();
                continue;
            }
        } else if len != 0 && ! obj.replace.clone().unwrap().all_replace {
            obj.text = [&obj.text[..start], &string_to[0..], &obj.text[start+len..] ].concat();
            return true;
        }

        if len != 0 {
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
