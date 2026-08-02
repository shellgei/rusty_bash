//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::utils::glob;
use crate::utils::glob::GlobElem;
use crate::elements::word::{Word, WordMode};
use super::{BracedParamExtension, ExecError, ParseError, Parameter};

#[derive(Debug, Clone, Default)]
pub struct Replace {
    pub text: String,
    pub symbol: String,
    pub pattern: Word,
    pub string: Word,
}

impl BracedParamExtension for Replace {
    fn exec(&mut self, v: &Parameter, text: &str, core: &mut ShellCore)
        -> Result<String, ExecError> {
        if ! core.db.exist(&v.text) {
            return Ok("".to_string());
        }

        let pat_str = self.pattern.eval_as_pattern(core)?;
        let pat = glob::parse(&pat_str);
        let string_to = self.string.eval_as_pattern(core)?;

        if self.symbol == "/" || self.symbol == "//" {
            return Ok(self.replace(text, &pat, &string_to));
        }

        Ok(text.to_string())
    }

    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> {
        Box::new(self.clone())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl Replace {
    pub fn replace(&self, text: &str, pattern: &[GlobElem],
                   string_to: &str) -> String {
        let mut ans = String::new();
        let mut start = 0;
        let mut first = true;
        let mut skip = 0;
     
        for ch in text.chars() {
            if !first && self.symbol == "/" {
                return ans + &text[start..];
            }

            if skip > 0 {
                skip -= ch.len_utf8();
                continue;
            }

            let s = text[start..].to_string();
            let len = glob::match_length(&s, &pattern, true);
            if len != 0 {
                ans.push_str(string_to);
                skip = len - ch.len_utf8();
                first = false;
                start += len;
            }else{
                ans.push(ch);
                start += ch.len_utf8();
            }
        }   

        ans
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
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

        if !feeder.starts_with("/") { //79〜86行目まで追加
            return Ok(Some(ans));
        }
        ans.text += &feeder.consume(1);
    
        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string()]));
        ans.string = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &ans.string.text.clone();

        Ok(Some(ans))
    }
}
