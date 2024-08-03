//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use super::Word;

pub fn eval(word: &mut Word) -> Vec<Word> {
    for i in open_brace_pos(word) {
        let d = parse(&word.subwords[i..], i);
        if d.len() > 2 {
            return expand(&word.subwords, &d);
        }
    }
    vec![word.clone()]
}

fn open_brace_pos(w: &Word) -> Vec<usize> {
    w.subwords.iter()
        .enumerate()
        .filter(|e| e.1.get_text() == "{")
        .map(|e| e.0)
        .collect()
}

fn parse(_: &[Box<dyn Subword>], _: usize) -> Vec<usize> { vec![] }
fn expand(_: &Vec<Box<dyn Subword>>, _: &Vec<usize>) -> Vec<Word> { vec![] }
