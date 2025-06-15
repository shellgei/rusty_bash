//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::elements::word::{Word, WordMode};
use crate::utils::glob;
use crate::utils::glob::GlobElem;
use crate::error::parse::ParseError;
use super::super::Variable;
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
    fn get_text(&self) -> String {self.text.clone()}
    fn exec(&mut self, param: &Variable, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        match core.db.exist(&param.name) {
            true  => self.get_text(text, core),
            false => Ok("".to_string()),
        }
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {Box::new(self.clone())}

    fn set_array(&mut self, param: &Variable, array: &mut Vec<String>,
                    text: &mut String, core: &mut ShellCore) -> Result<(), ExecError> {
        *array = match param.name.as_str() {
            "@" | "*" => core.db.get_position_params(),
            _ => core.db.get_vec(&param.name, true)?,
        };

        for i in 0..array.len() {
            array[i] = self.get_text(&array[i], core)?;
        }

        if param.name == "@"
        || (param.index.is_some() && param.index.as_ref().unwrap().text == "[@]") {
            *text = array.join(" ");
            return Ok(());
        }

        let ifs = core.db.get_ifs_head();
        *text = array.join(&ifs);
        Ok(())
    }

    fn has_array_replace(&self) -> bool {true}
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

        if let Some(w) = Word::parse(feeder, core, Some(WordMode::ParamOption(vec!["}".to_string(), "/".to_string()])))? {
            ans.text += &w.text.clone();
            ans.replace_from = Some(w);
        }else{
            ans.replace_from = Some(Word::default());
        }

        if ! feeder.starts_with("/") {
            return Ok(Some(ans));
        }
        ans.text += &feeder.consume(1);
        ans.has_replace_to = true;

        if let Some(w) = Word::parse(feeder, core, Some(WordMode::ParamOption(vec!["}".to_string()])))? {
            ans.text += &w.text.clone();
            ans.replace_to = Some(w);
        }else{
            ans.replace_to = Some(Word::default());
        }

        Ok(Some(ans))
    }
}
