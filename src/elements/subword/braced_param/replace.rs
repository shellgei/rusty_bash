//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::elements::subword::braced_param::Word;
use crate::utils::glob;
use crate::error::ParseError;
use super::BracedParam;

#[derive(Debug, Clone, Default)]
pub struct Replace {
    pub head_only_replace: bool,
    pub tail_only_replace: bool,
    pub all_replace: bool,
    pub replace_from: Option<Word>,
    pub replace_to: Option<Word>,
    pub has_replace_to: bool,
}

impl Replace {
    fn to_string(&self, w: &Option<Word>, core: &mut ShellCore) -> Result<String, ExecError> {
        if let Some(w) = &w {
            match w.eval_for_case_word(core) {
                Some(s) => return Ok(s),
                None => match w.subwords.len() {
                    0 => return Ok("".to_string()),
                    _ => return Err(ExecError::Other("parse error".to_string())),
                },
            }
        }

        Err(ExecError::Other("parse error".to_string()))
    }

    pub fn get_text(&self, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        let pattern = self.to_string(&self.replace_from, core)?;
        let string_to = self.to_string(&self.replace_to, core)?;
        let extglob = core.shopts.query("extglob");
    
        let mut start = 0;
        let mut ans = String::new();
        let mut skip = 0;
        for ch in text.chars() {
            if start != 0 && self.head_only_replace {
                return Ok(text.clone());
            }
            if skip > 0 {
                skip -= 1;
                start += ch.len_utf8();
                continue;
            }
    
            let pat = glob::parse(&pattern, extglob);
            let len = glob::longest_match_length(&text[start..].to_string(), &pat);
            if len != 0 && self.tail_only_replace {
                if len == text[start..].len() {
                    return Ok([&text[..start], &string_to[0..] ].concat());
                }else{
                    ans += &ch.to_string();
                    start += ch.len_utf8();
                    continue;
                }
            } else if len != 0 && ! self.all_replace {
                return Ok([&text[..start], &string_to[0..], &text[start+len..] ].concat());
            }
    
            if len != 0 {
                skip = text[start..start+len].chars().count() - 1;
                ans += &string_to.clone();
            }else{
                ans += &ch.to_string();
            }
            start += ch.len_utf8();
        }
    
        Ok(ans)
    }

    pub fn eat(feeder: &mut Feeder, ans: &mut BracedParam, core: &mut ShellCore)
           -> Result<bool, ParseError> {
        if ! feeder.starts_with("/") {
            return Ok(false);
        }

        let mut info = Replace::default();

        ans.text += &feeder.consume(1);
        if feeder.starts_with("/") {
            ans.text += &feeder.consume(1);
            info.all_replace = true;
        }else if feeder.starts_with("#") {
            ans.text += &feeder.consume(1);
            info.head_only_replace = true;
        }else if feeder.starts_with("%") {
            ans.text += &feeder.consume(1);
            info.tail_only_replace = true;
        }

        info.replace_from = Some(BracedParam::eat_subwords(feeder, ans, vec!["}", "/"], core)? );

        if ! feeder.starts_with("/") {
            ans.replace = Some(info);
            return Ok(true);
        }
        ans.text += &feeder.consume(1);
        info.has_replace_to = true;
        info.replace_to = Some(BracedParam::eat_subwords(feeder, ans, vec!["}"], core)? );

        ans.replace = Some(info);
        Ok(true)
    }
}
