//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::BracedParamExtension;
use crate::elements::parameter::Parameter;
use crate::elements::word::{Word, mode::WordMode};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::glob;
use crate::{Feeder, ShellCore};

impl BracedParamExtension for Remove {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(
        &mut self,
        _: &Parameter,
        text: &str,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        self.set(text, core)
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
            *item = self.set(item, core)?;
        }

        *text = array.join(" ");
        Ok(())
    }

    fn has_array_replace(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Default)]
pub struct Remove {
    pub text: String,
    pub symbol: String,
    pub pattern: Word,
}

impl Remove {
    pub fn set(&mut self, text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut text = text.to_string();
        let pattern = self.pattern.eval_for_case_word(core)?;
        let extglob = core.shopts.query("extglob");

        if self.symbol.starts_with("##") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::longest_match_length(&text, &pat);
            text = text[len..].to_string();
        } else if self.symbol.starts_with("#") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::shortest_match_length(&text, &pat);
            text = text[len..].to_string();
        } else if self.symbol.starts_with("%") {
            self.percent(&mut text, &pattern, extglob);
        } else {
            return Err(ExecError::Other("unknown symbol".to_string()));
        }

        Ok(text)
    }

    pub fn percent(&self, text: &mut String, pattern: &str, extglob: bool) {
        let mut length = text.len();
        let mut ans_length = length;

        for ch in text.chars().rev() {
            length -= ch.len_utf8();
            let s = text[length..].to_string();

            if glob::parse_and_compare(&s, pattern, extglob) {
                ans_length = length;
                if self.symbol == "%" {
                    break;
                }
            }
        }

        *text = text[0..ans_length].to_string();
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_parameter_remove_symbol();
        if len == 0 {
            return Ok(None);
        }

        let mut ans = Remove::default();
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();

        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string()]));
        ans.pattern = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &ans.pattern.text.clone();
        Ok(Some(ans))
    }
}
