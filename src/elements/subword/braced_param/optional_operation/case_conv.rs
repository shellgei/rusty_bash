//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::super::Variable;
use super::OptionalOperation;
use crate::elements::word::{Word, WordMode};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::glob;
use crate::utils::glob::GlobElem;
use crate::{Feeder, ShellCore};

impl OptionalOperation for CaseConv {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(
        &mut self,
        _: &Variable,
        text: &str,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        self.get_text(text, core)
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CaseConv {
    pub text: String,
    pub pattern: Option<Word>,
    pub replace_symbol: String,
}

impl CaseConv {
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

    fn get_match_length(&self, text: &str, pattern: &[GlobElem], ch: char) -> usize {
        if pattern.is_empty() {
            return ch.len_utf8();
        }
        glob::longest_match_length(text, pattern)
    }

    fn conv(&self, ch: char) -> String {
        if ch.is_ascii_lowercase()
            && (self.replace_symbol.starts_with("^") || self.replace_symbol.starts_with("~"))
        {
            return ch.to_string().to_uppercase();
        }

        if ch.is_ascii_uppercase()
            && (self.replace_symbol.starts_with(",") || self.replace_symbol.starts_with("~"))
        {
            return ch.to_string().to_lowercase();
        }

        ch.to_string()
    }

    pub fn get_text(&self, text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
        let tmp = self.to_string(&self.pattern, core)?;
        let extglob = core.shopts.query("extglob");
        let pattern = glob::parse(&tmp, extglob);

        let mut start = 0;
        let mut ans = String::new();
        let mut skip = 0;
        for ch in text.chars() {
            if skip > 0 {
                skip -= 1;
                start += ch.len_utf8();
                continue;
            }

            let len = self.get_match_length(&text[start..], &pattern, ch);
            if len == 0 {
                ans += &ch.to_string();
                start += ch.len_utf8();
                continue;
            }

            let new_ch = self.conv(ch);
            ans += &new_ch;
            if self.replace_symbol.len() != 2 {
                return Ok(ans + &text[start + len..]);
            }

            start += ch.len_utf8();
        }
        Ok(ans)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("^") && !feeder.starts_with(",") && !feeder.starts_with("~") {
            return Ok(None);
        }

        let mut ans = CaseConv::default();

        if feeder.starts_with("^^") || feeder.starts_with(",,") || feeder.starts_with("~~") {
            ans.replace_symbol = feeder.consume(2);
            ans.text += &ans.replace_symbol;
        } else if feeder.starts_with("^") || feeder.starts_with(",") || feeder.starts_with("~") {
            ans.replace_symbol = feeder.consume(1);
            ans.text += &ans.replace_symbol;
        }

        if let Some(w) = Word::parse(
            feeder,
            core,
            Some(WordMode::ParamOption(vec!["}".to_string()])),
            //Some(WordMode::AlterWord),
        )? {
            ans.text += &w.text.clone();
            ans.pattern = Some(w);
        } else {
            ans.pattern = Some(Word::default());
        }

        Ok(Some(ans))
    }
}
