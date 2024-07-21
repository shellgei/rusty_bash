//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;

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
        let mut ws = brace_expansion::eval(self);
        for w in ws.iter_mut() {
            eprint!("parse of {}: ", &w.text);
            w.subwords.iter_mut().for_each(|sw| {
                match sw.is_name() {
                    true  => eprint!("NAME"),
                    false => eprint!("{}", sw.get_text()),
                }
                sw.substitute(core);
            } );
            eprintln!("");
        }
        Some( Self::make_args(&mut ws) )
    }

    pub fn make_args(words: &mut Vec<Word>) -> Vec<String> {
        words.iter_mut()
              .map(|w| w.make_unquoted_word())
              .filter(|w| *w != None)
              .map(|w| w.unwrap())
              .collect()
    }

    pub fn make_unquoted_word(&mut self) -> Option<String> {
        let sw: Vec<Option<String>> = self.subwords.iter_mut()
            .map(|s| s.make_unquoted_string()) //""や''はNoneにならずに空文字として残る
            .filter(|s| *s != None)
            .collect();

        if sw.len() == 0 {
            return None;
        }

        Some(sw.into_iter().map(|s| s.unwrap()).collect::<String>())
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
