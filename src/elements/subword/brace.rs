//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;
use crate::elements::word::Word;

#[derive(Debug)]
pub struct BraceSubword {
    text: String,
    words: Vec<Word>,
}

impl Subword for BraceSubword {
    fn get_text(&self) -> String { self.text.clone() }

    fn copy(&self) -> Box<dyn Subword> {
        Box::new( BraceSubword { 
            text: self.text.clone(),
            words: self.words.iter().map(|w| w.copy()).collect(),
        } )
    }   

    fn brace_expansion(&mut self, lefts: &mut Vec<Word>) {
        let mut rights = vec![];
        for w in self.words.iter_mut() {
            rights.extend(w.brace_expansion());
        }

        let mut ans = vec![];
        for lf in lefts.iter_mut() {
            ans.extend(self.add_expended(lf, &rights));
        }
        *lefts = ans;
    }
}

impl BraceSubword {
    fn new() -> BraceSubword {
        BraceSubword {
            text: String::new(),
            words: vec![],
        }
    }

    fn add_expended(&mut self, left: &mut Word, rights: &Vec<Word>) -> Vec<Word> {
        if self.words.len() < 2 { 
            left.add_text("{");
        }

        let mut ans = vec![];
        for rw in rights {
            let mut lw = left.copy();
            lw.concat(&rw);
            ans.push(lw);
        }

        if self.words.len() < 2 { 
            ans.iter_mut().for_each(|w| w.add_text("}"));
        }

        ans
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut BraceSubword, core: &mut ShellCore) -> bool {
        match Word::parse(feeder, core) {
            Some(w) => {
                ans.text += &w.text;
                ans.words.push(w);
                true
            },
            _ => {
                if feeder.starts_with(",") || feeder.starts_with("}") {
                    ans.words.push(Word::new());
                    return true;
                }
                false
            },
        }
    } 

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BraceSubword> {
       if ! feeder.starts_with("{") {
            return None;
        }
        feeder.set_backup();
        core.word_nest.push("{".to_string());

        let mut ans = Self::new();
        let mut closed = false;

        ans.text = feeder.consume(1); // {

        while Self::eat_word(feeder, &mut ans, core) {
            if feeder.starts_with(",") {
                ans.text += &feeder.consume(1);
                core.word_nest.push(",".to_string());
                continue;
            }

            if feeder.starts_with("}") {
                ans.text += &feeder.consume(1);
                closed = true;
            }
            break;
        }

        while core.word_nest.last().unwrap() == "," {
            core.word_nest.pop();
        }
        core.word_nest.pop();

        if closed && ans.words.len() >= 1 {
            feeder.pop_backup();
            ans.text.insert(0, '<');
            ans.text.push('>');
            Some(ans)
        }else{
            feeder.rewind();
            None
        }
    }
}
