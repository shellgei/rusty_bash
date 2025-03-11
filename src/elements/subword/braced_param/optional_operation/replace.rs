//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::elements::subword::braced_param::Word;
use crate::utils::glob;
use crate::utils::glob::GlobElem;
use crate::error::parse::ParseError;
use super::super::{BracedParam, Param};
use super::super::optional_operation::OptionalOperation;

#[derive(Debug, Clone, Default)]
pub struct Replace {
    pub text: String,
    pub head_only_replace: bool,
    pub tail_only_replace: bool,
    pub all_replace: bool,
    pub replace_from: Option<Word>,
    pub replace_to: Option<Word>,
    pub has_replace_to: bool,
}

impl OptionalOperation for Replace {
    fn exec(&mut self, param: &Param, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        match core.db.has_value(&param.name) {
            true  => self.get_text(text, core),
            false => Ok("".to_string()),
        }
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {Box::new(self.clone())}
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

        Ok("".to_string())
    }

    fn get_text_head(text: &String, pattern: &Vec<GlobElem>, string_to: &String) -> Result<String, ExecError> {
        let len = glob::longest_match_length(text, pattern);
        if len == 0 && ! pattern.is_empty() {
            return Ok(text.clone());
        }

        let ans = string_to.clone() + &text[len..];
        Ok(ans)
    }

    fn get_text_tail(text: &String, pattern: &Vec<GlobElem>, string_to: &String) -> Result<String, ExecError> {
        if pattern.is_empty() {
            let ans = text.to_string() + &string_to;
            return Ok(ans);
        }

        let mut start = 0;
        for ch in text.chars() {
            let len = glob::longest_match_length(&text[start..].to_string(), pattern);

            if len == text[start..].len() {
                let ans = text[..start].to_string() + &string_to;
                return Ok(ans);
            }

            start += ch.len_utf8();
        }

        Ok(text.to_string())
    }

    pub fn get_text(&self, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        let extglob = core.shopts.query("extglob");
        let tmp = self.to_string(&self.replace_from, core)?;
        let pattern = glob::parse(&tmp, extglob);
        let string_to = self.to_string(&self.replace_to, core)?;

        if self.head_only_replace {
            return Self::get_text_head(text, &pattern, &string_to);
        }else if self.tail_only_replace {
            return Self::get_text_tail(text, &pattern, &string_to);
        }

        let mut start = 0;
        let mut ans = String::new();
        let mut skip = 0;
        for ch in text.chars() {
            if skip > 0 {
                skip -= 1;
                start += ch.len_utf8();
                continue;
            }
    
            let len = glob::longest_match_length(&text[start..].to_string(), &pattern);
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

    /*
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
            ans.optional_operation = Some(Box::new(info));
            return Ok(true);
        }
        ans.text += &feeder.consume(1);
        info.has_replace_to = true;
        info.replace_to = Some(BracedParam::eat_subwords(feeder, ans, vec!["}"], core)? );

        ans.optional_operation = Some(Box::new(info));
        Ok(true)
    }*/

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("/") {
            return Ok(None);
        }

        let mut ans = Replace::default();

        ans.text += &feeder.consume(1);
        if feeder.starts_with("/") {
            ans.text += &feeder.consume(1);
            ans.all_replace = true;
        }else if feeder.starts_with("#") {
            ans.text += &feeder.consume(1);
            ans.head_only_replace = true;
        }else if feeder.starts_with("%") {
            ans.text += &feeder.consume(1);
            ans.tail_only_replace = true;
        }

        ans.replace_from = Some(BracedParam::eat_subwords2(feeder, vec!["}", "/"], core)? );

        if ! feeder.starts_with("/") {
        //    ans.optional_operation = Some(Box::new(ans));
            return Ok(Some(ans));
        }
        ans.text += &feeder.consume(1);
        ans.has_replace_to = true;
        ans.replace_to = Some(BracedParam::eat_subwords2(feeder, vec!["}"], core)? );

        //ans.optional_operation = Some(Box::new(ans));
        Ok(Some(ans))
    }
}
