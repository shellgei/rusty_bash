//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::subword::braced_param::Word;
use crate::utils::glob;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use super::super::{BracedParam, Param};
use super::OptionalOperation;

impl OptionalOperation for Remove {
    fn get_text(&self) -> String {self.text.clone()}
    fn exec(&mut self, _: &Param, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        self.set(text, core)
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {Box::new(self.clone())}
}

#[derive(Debug, Clone, Default)]
pub struct Remove {
    pub text: String,
    pub remove_symbol: String,
    pub remove_pattern: Option<Word>,
}

impl Remove {
    pub fn set(&mut self, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut text = text.clone();
        let pattern = self.remove_pattern.as_mut().unwrap()
                            .eval_for_case_word(core).ok_or(ExecError::Other("evaluation error".to_string()))?;
        let extglob = core.shopts.query("extglob");
     
        if self.remove_symbol.starts_with("##") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::longest_match_length(&text, &pat);
            text = text[len..].to_string();
        } else if self.remove_symbol.starts_with("#") {
            let pat = glob::parse(&pattern, extglob);
            let len = glob::shortest_match_length(&text, &pat);
            text = text[len..].to_string();
        }else if self.remove_symbol.starts_with("%") {
            self.percent(&mut text, &pattern, extglob);
        }else {
            return Err(ExecError::Other("unknown symbol".to_string()));
        }

        Ok(text)
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

    /*
    pub fn eat(feeder: &mut Feeder, ans: &mut BracedParam, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let len = feeder.scanner_parameter_remove_symbol();
        if len == 0 {
            return Ok(false);
        }

        let mut ans = Remove::default();

        ans.remove_symbol = feeder.consume(len);
        ans.text += &ans.remove_symbol.clone();

        ans.remove_pattern = Some(BracedParam::eat_subwords(feeder, ans, vec!["}"], core)? );
        ans.optional_operation = Some(Box::new(ans));
        Ok(true)
    }*/

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_parameter_remove_symbol();
        if len == 0 {
            return Ok(None);
        }

        let mut ans = Remove::default();

        ans.remove_symbol = feeder.consume(len);
        ans.text += &ans.remove_symbol.clone();

        ans.remove_pattern = Some(BracedParam::eat_subwords(feeder, vec!["}"], core)? );
        Ok(Some(ans))
    }
}
