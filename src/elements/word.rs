//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;
mod tilde_expansion;
mod substitution;

use crate::{Feeder, ShellCore};
use crate::elements::subword;
use super::subword::Subword;

#[derive(Debug, Clone)] //Cloneも指定しておく
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

    fn make_args(words: &mut Vec<Word>) -> Vec<String> {
        words.iter_mut()
              .map(|w| w.make_unquoted_word())
              .filter(|w| *w != None)
              .map(|w| w.unwrap())
              .collect()
    }

    fn make_unquoted_word(&mut self) -> Option<String> {
        let sw: Vec<Option<String>> = self.subwords.iter_mut()
            .map(|s| s.make_unquoted_string()) //""や''はNoneにならずに空文字として残る
            .filter(|s| *s != None)
            .collect();

        if sw.len() == 0 {
            return None;
        }

        Some(sw.into_iter().map(|s| s.unwrap()).collect::<String>())
    }

    fn scan_pos(&self, s: &str) -> Vec<usize> {
        self.subwords.iter()
            .enumerate()
            .filter(|e| e.1.get_text() == s)
            .map(|e| e.0)
            .collect()
    }

    pub fn new(subwords: Vec<Box::<dyn Subword>>) -> Word {
        Word {
            text: subwords.iter().map(|s| s.get_text()).collect(),
            subwords: subwords,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Word> {
        if feeder.starts_with("#") {
            return None;
        }

        let mut subwords = vec![];
        while let Some(sw) = subword::parse(feeder, core) {
            subwords.push(sw);
        }

        let ans = Word::new(subwords);
        match ans.text.len() {
            0 => None,
            _ => Some(ans),
        }
    }
}
