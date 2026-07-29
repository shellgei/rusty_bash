//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::BracedParamExtension;
use crate::elements::parameter::Parameter;
use crate::elements::word::{Word, mode::WordMode};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::glob;
use crate::utils::glob::GlobElem;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct Replace {
    pub text: String,
    pub symbol: String,
    pub pattern: Word,
    pub string: Word,
}

impl BracedParamExtension for Replace {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(
        &mut self,
        param: &Parameter,
        text: &str,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        match core.db.exist(&param.name) {
            true => self.get_text(text, core),
            false => Ok("".to_string()),
        }
    }

    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> {
        Box::new(self.clone())
    }

    fn init_array(
        &mut self,
        param: &Parameter,
        array: &mut Vec<String>,
        text: &mut String,
        core: &mut ShellCore,
    ) -> Result<(), ExecError> {
        *array = match param.name.as_str() {
            "@" | "*" => core.db.get_position_params(),
            _ => core.db.get_vec(&param.name, true)?,
        };

        for item in array.iter_mut() {
            *item = self.get_text(item, core)?;
        }

        if param.name == "@"
            || (param.index.is_some() && param.index.as_ref().unwrap().text == "[@]")
        {
            *text = array.join(" ");
            return Ok(());
        }

        let ifs = core.db.get_ifs_head();
        *text = array.join(&ifs);
        Ok(())
    }

    fn has_array_replace(&self) -> bool {
        true
    }
}

impl Replace {
    fn get_text_head(text: &str,
        pattern: &[GlobElem],
        string_to: &str,
    ) -> Result<String, ExecError> {
        let len = glob::longest_match_length(text, pattern);
        if len == 0 && !pattern.is_empty() {
            return Ok(text.to_string());
        }

        let ans = string_to.to_string() + &text[len..];
        Ok(ans)
    }

    fn get_text_tail(
        text: &str,
        pattern: &[GlobElem],
        string_to: &str,
    ) -> Result<String, ExecError> {
        if pattern.is_empty() {
            let ans = text.to_string() + string_to;
            return Ok(ans);
        }

        let mut start = 0;
        for ch in text.chars() {
            let len = glob::longest_match_length(&text[start..], pattern);

            if len == text[start..].len() {
                let ans = text[..start].to_string() + string_to;
                return Ok(ans);
            }

            start += ch.len_utf8();
        }

        Ok(text.to_string())
    }

    pub fn get_text(&self, text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
        let extglob = core.shopts.query("extglob");
        let nocasematch = core.shopts.query("nocasematch");
        let tmp = self.pattern.eval_as_pattern(core)?;
        let pattern = glob::parse(&tmp, extglob, nocasematch);
        let string_to = self.string.eval_as_pattern(core)?;

        if self.symbol == "/#" {
            return Self::get_text_head(text, &pattern, &string_to);
        } else if self.symbol == "/%" {
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

            let len = glob::longest_match_length(&text[start..], &pattern);
            if len != 0 && self.symbol == "/%" {
                if len == text[start..].len() {
                    return Ok([&text[..start], &string_to[0..]].concat());
                } else {
                    ans += &ch.to_string();
                    start += ch.len_utf8();
                    continue;
                }
            } else if len != 0 && self.symbol != "//" {
                return Ok([&text[..start], &string_to[0..], &text[start + len..]].concat());
            }

            if len != 0 {
                skip = text[start..start + len].chars().count() - 1;
                ans += &string_to.clone();
            } else {
                ans += &ch.to_string();
            }
            start += ch.len_utf8();
        }

        Ok(ans)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_parameter_replace_symbol();
        if len == 0 {
            return Ok(None);
        }
        
        let mut ans = Replace::default();
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();

        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string(), "/".to_string()]));
        ans.pattern = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &ans.pattern.text.clone();

        if !feeder.starts_with("/") {
            return Ok(Some(ans));
        }
        ans.text += &feeder.consume(1);

        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string()]));
        ans.string = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &ans.string.text.clone();

        Ok(Some(ans))
    }
}
