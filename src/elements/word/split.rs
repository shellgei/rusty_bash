//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::Subword;

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    let (left_sws, mut center_sws, right_sws) = match find_and_split(word) {
        (Some(i), sws) => (&word.subwords[..i], sws, &word.subwords[i+1..]),
        (None, _)      => return vec![word.clone()],
    };

    let left = Word::from([ left_sws, &[center_sws.remove(0)] ].concat());
    let right = Word::from([ &[center_sws.pop().unwrap()], right_sws].concat());
    let centers = center_sws.iter().map(|s| Word::from(s.clone())).collect();

    [ vec![left], centers, eval(&right, core) ].concat()
}

pub fn find_and_split(word: &Word) -> (Option<usize>, Vec<Box::<dyn Subword>>) {
    for (i, sw) in word.subwords.iter().enumerate() {
        let subwords = sw.split();
        if subwords.len() >= 2 {
            return (Some(i), subwords);
        }
    }
    (None, vec![])
}
