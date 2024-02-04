//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use crate::elements::word::Word;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let mut start_pos = vec![];
    for sw in word.subwords.iter().enumerate().filter(|e| e.1.get_text() == "{") {
        start_pos.push(sw.0);
    }
    if word.text.starts_with("{}") {
        start_pos.remove(0);
    }

    for i in start_pos {
        if let Some(d) = parse(&word.subwords[i..]) {
            let shift_d = d.iter().map(|e| e+i).collect();
            return expand(&word.subwords, &shift_d);
        }
    }

    vec![word.clone()]
}

pub fn parse(_: &[Box<dyn Subword>]) -> Option<Vec<usize>> {
    None
}

pub fn expand(_: &Vec<Box<dyn Subword>>, _: &Vec<usize>) -> Vec<Word> {
    vec![]
}

