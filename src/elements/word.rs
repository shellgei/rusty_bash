//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;
mod tilde_expansion;
pub mod substitution;
mod path_expansion;
mod split;

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::subword;
use super::subword::Subword;
use super::subword::simple::SimpleSubword;

#[derive(Debug, Clone, Default)]
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl From<&str> for Word {
    fn from(text: &str) -> Self {
        Self::from(
            Box::new(
                SimpleSubword{ text: text.to_string() }
            ) as Box::<dyn Subword>
       )
    }
}

impl From<Box::<dyn Subword>> for Word {
    fn from(subword: Box::<dyn Subword>) -> Self {
        Self {
            text: subword.get_text().to_string(),
            subwords: vec![subword],
        }
    }
}

impl From<Vec<Box::<dyn Subword>>> for Word {
    fn from(subwords: Vec<Box::<dyn Subword>>) -> Self {
        Self {
            text: subwords.iter().map(|s| s.get_text()).collect(),
            subwords,
        }
    }
}

impl Word {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<Vec<String>, ExecError> {
        let mut ws = vec![];
        for w in brace_expansion::eval(&mut self.clone()) {
            let expanded = w.tilde_and_dollar_expansion(core)?;
            ws.append( &mut expanded.split_and_path_expansion() );
        }
        Self::make_args(&mut ws)
    }

    pub fn tilde_and_dollar_expansion(&self, core: &mut ShellCore) -> Result<Word, ExecError> {
        let mut w = self.clone();
        tilde_expansion::eval(&mut w, core);
        substitution::eval(&mut w, core)?;
        Ok(w)
    }

    pub fn split_and_path_expansion(&self) -> Vec<Word> {
        let mut ans = vec![];
        for mut w in split::eval(self) {
            ans.append(&mut path_expansion::eval(&mut w) );
        }
        ans
    }

    fn make_args(words: &mut [Word]) -> Result<Vec<String>, ExecError> {
        Ok( words.iter_mut().filter_map(|w| w.make_unquoted_word()).collect() )
    }

    fn make_unquoted_word(&mut self) -> Option<String> {
        let sw: Vec<Option<String>> = self.subwords.iter_mut()
            .map(|s| s.make_unquoted_string()) //""や''はNoneにならずに空文字として残る
            .filter(|s| s.is_some())
            .collect();

        if sw.is_empty() {
            return None;
        }

        Some(sw.into_iter().map(|s| s.unwrap()).collect::<String>())
    }

    fn make_glob_string(&mut self) -> String {
        self.subwords.iter_mut()
            .map(|s| s.make_glob_string())
            .collect::<Vec<String>>()
            .concat()
    }

    fn scan_pos(&self, s: &str) -> Vec<usize> {
        self.subwords.iter()
            .enumerate()
            .filter(|e| e.1.get_text() == s)
            .map(|e| e.0)
            .collect()
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Word>, ParseError> {
        if feeder.starts_with("#") {
            return Ok(None);
        }

        let mut subwords = vec![];
        while let Some(sw) = subword::parse(feeder, core)? {
            subwords.push(sw);
        }

        let ans = Word::from(subwords);
        match ans.text.len() {
            0 => Ok(None),
            _ => Ok(Some(ans)),
        }
    }
}
