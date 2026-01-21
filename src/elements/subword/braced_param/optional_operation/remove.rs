//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::super::Variable;
use super::OptionalOperation;
use crate::elements::word::{Word, WordMode};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::glob;
use crate::{Feeder, ShellCore};

impl OptionalOperation for Remove {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(
        &mut self,
        _: &Variable,
        text: &str,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        self.set(text, core)
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {
        Box::new(self.clone())
    }

    fn init_array(
        &mut self,
        param: &Variable,
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
    pub remove_symbol: String,
    pub remove_pattern: Option<Word>,
}

impl Remove {
    pub fn set(&mut self, text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut text = text.to_string();
        let pattern = self
            .remove_pattern
            .as_mut()
            .unwrap()
            .eval_for_case_word(core)
            .ok_or(ExecError::Other("evaluation error".to_string()))?;
        let extglob = core.shopts.query("extglob");

        if self.remove_symbol.starts_with("##") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::longest_match_length(&text, &pat);
            text = text[len..].to_string();
        } else if self.remove_symbol.starts_with("#") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::shortest_match_length(&text, &pat);
            text = text[len..].to_string();
        } else if self.remove_symbol.starts_with("%") {
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
                if self.remove_symbol == "%" {
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

        let mut ans = Remove {
            remove_symbol: feeder.consume(len),
            ..Default::default()
        };
        ans.text += &ans.remove_symbol.clone();

        if let Some(w) = Word::parse(
            feeder,
            core,
            Some(WordMode::ParamOption(vec!["}".to_string()])),
            //Some(WordMode::AlterWord),
        )? {
            ans.text += &w.text.clone();
            ans.remove_pattern = Some(w);
        } else {
            ans.remove_pattern = Some(Word::default());
        }
        Ok(Some(ans))
    }
}
