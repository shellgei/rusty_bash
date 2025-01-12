//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::braced_param::Word;
use crate::utils::glob;

#[derive(Debug, Clone, Default)]
pub struct Remove {
    pub remove_symbol: String,
    pub remove_pattern: Option<Word>,
}

impl Remove {
    pub fn set(&self, text: &mut String, core: &mut ShellCore) -> bool {
        let pattern = match &self.remove_pattern {
            None => return true,
            Some(w) => {
                match w.eval_for_case_word(core) {
                    Some(s) => s,
                    None    => return false,
                }
            },
        };
     
        let extglob = core.shopts.query("extglob");
     
        if self.remove_symbol.starts_with("##") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::longest_match_length(&text, &pat);
            *text = text[len..].to_string();
        } else if self.remove_symbol.starts_with("#") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::shortest_match_length(&text, &pat);
            *text = text[len..].to_string();
        }else if self.remove_symbol.starts_with("%") {
            self.percent(text, &pattern, extglob);
        }else {
            return false;
        }
        true
    }
    
    pub fn percent(&self, text: &mut String, pattern: &String, extglob: bool) {
        let mut length = text.len();
        let mut ans_length = length;
     
        for ch in text.chars().rev() {
            length -= ch.len_utf8();
            let s = text[length..].to_string();
     
            if glob::parse_and_compare(&s, &pattern, extglob) {
                ans_length = length;
                if self.remove_symbol == "%" {
                    break;
                }
            }
        }
     
        *text = text[0..ans_length].to_string();
    }
}
