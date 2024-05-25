//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;
mod tilde_expansion;
pub mod substitution;
mod path_expansion;
mod split;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use super::subword::Subword;

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Word {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<Vec<String>> {
        let mut ws = vec![];
        for w in brace_expansion::eval(&mut self.clone()) {
            match w.tilde_and_dollar_expansion(core) {
                Some(w) => ws.append( &mut w.split_and_path_expansion(core) ),
                None    => return None,
            };
        }

        Some( Self::make_args(&mut ws) )
    }

    pub fn eval_as_value(&self, core: &mut ShellCore) -> Option<String> {
        let mut ws = match self.tilde_and_dollar_expansion(core) {
            Some(w) => w.split_and_path_expansion(core),
            None    => return None,
        };

        Some( Self::make_args(&mut ws).join(" ") )
    }

    pub fn eval_for_case_word(&self, core: &mut ShellCore) -> Option<String> {
        match self.tilde_and_dollar_expansion(core) {
            Some(mut w) => Some(w.make_unquoted_word()),
            None    => return None,
        }
    }

    pub fn eval_for_case_pattern(&self, core: &mut ShellCore) -> Option<String> {
        match self.tilde_and_dollar_expansion(core) {
            Some(mut w) => Some(w.make_glob_string()),
            None    => return None,
        }
    }

    pub fn tilde_and_dollar_expansion(&self, core: &mut ShellCore) -> Option<Word> {
        let mut w = self.clone();
        tilde_expansion::eval(&mut w, core);
        match substitution::eval(&mut w, core) {
            true  => Some(w),
            false => None,
        }
    }

    pub fn split_and_path_expansion(&self, core: &mut ShellCore) -> Vec<Word> {
        let mut ans = vec![];
        for mut w in split::eval(self, core) {
            ans.append(&mut path_expansion::eval(&mut w) );
        }
        ans
    }

    pub fn make_args(words: &mut Vec<Word>) -> Vec<String> {
        words.iter_mut()
              .map(|w| w.make_unquoted_word())
              .filter(|arg| arg.len() > 0)
              .collect()
    }

    pub fn make_unquoted_word(&mut self) -> String {
        self.subwords.iter_mut().map(|s| s.make_unquoted_string()).collect::<String>()
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

    pub fn new() -> Word {
        Word {
            text: String::new(),
            subwords: vec![],
        }
    }

    fn push(&mut self, subword: &Box<dyn Subword>) {
        self.text += &subword.get_text().to_string();
        self.subwords.push(subword.clone());
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        if feeder.starts_with("#") {
            return None;
        }

        let mut ans = Word::new();
        while let Some(sw) = subword::parse(feeder, core) {
            ans.push(&sw);
        }

        if ans.text.len() == 0 {
            None
        }else{
            Some(ans)
        }
    }
}
