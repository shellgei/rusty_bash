//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;
mod tilde_expansion;
pub mod substitution;
mod path_expansion;
mod split;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Word {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<Vec<String>> {
        let mut ws = brace_expansion::eval(self);

        ws.iter_mut().for_each(|w| tilde_expansion::eval(w, core));
        if ! ws.iter_mut().all(|w| substitution::eval(w, core)) {
            return None;
        }
        ws = itertools::concat(ws.iter_mut().map(|w| split::eval(w, core)) );
        ws.iter_mut().for_each(|w| w.connect_subwords());
        ws = itertools::concat(ws.iter_mut().map(|w| path_expansion::eval(w)) );
        ws.iter_mut().for_each(|w| w.unquote());
        ws.iter_mut().for_each(|w| w.connect_subwords());
        let ans = ws.iter().map(|w| w.text.clone()).filter(|arg| arg.len() > 0).collect();

        Some(ans)
    }

    fn unquote(&mut self) {
        self.subwords.iter_mut().for_each(|w| w.unquote());
    }
    
    fn connect_subwords(&mut self) {
        self.text = self.subwords.iter()
                    .map(|s| s.get_text())
                    .collect::<String>();
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
