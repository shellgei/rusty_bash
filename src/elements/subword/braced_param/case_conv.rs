//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::subword::braced_param::Word;
use crate::error::parse::ParseError;
use super::BracedParam;

#[derive(Debug, Clone, Default)]
pub struct CaseConv {
    pub all_replace: bool,
    pub pattern: Option<Word>,
    pub to_upper: bool,
}

impl CaseConv {
/*
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

        Ok("".to_string())
        //Err(ExecError::Other("parse error".to_string()))
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
*/

    pub fn eat(feeder: &mut Feeder, ans: &mut BracedParam, core: &mut ShellCore)
           -> Result<bool, ParseError> {
        if ! feeder.starts_with("^") && ! feeder.starts_with(",") {
            return Ok(false);
        }

        let mut info = CaseConv::default();

        if feeder.starts_with("^^") {
            info.to_upper = true;
            info.all_replace = true;
            ans.text += &feeder.consume(2);
        }else if feeder.starts_with("^") {
            info.to_upper = true;
            ans.text += &feeder.consume(1);
        }else if feeder.starts_with(",,") {
            info.all_replace = true;
            ans.text += &feeder.consume(2);
        }else if feeder.starts_with(",") {
            ans.text += &feeder.consume(1);
        }

        info.pattern = Some(BracedParam::eat_subwords(feeder, ans, vec!["}"], core)? );
        ans.case_conv = Some(info);
        return Ok(true);
    }
}
