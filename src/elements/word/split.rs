//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::Subword;

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    let (pos, mut subws) = split(word);
    if subws.len() == 0 {
        return vec![word.clone()];
    }
    let (lsubws, rsubws) = (&word.subwords[..pos], &word.subwords[pos+1..]);

    let left = Word::from([ lsubws, &[subws.remove(0)] ].concat());
    let right = Word::from([ &[subws.pop().unwrap()], rsubws].concat());
    let centers = subws.iter().map(|s| Word::from(s.clone())).collect();

    [ vec![left], centers, eval(&right, core) ].concat()
}

pub fn split(word: &Word) -> (usize, Vec<Box::<dyn Subword>>) {
    for (i, sw) in word.subwords.iter().enumerate() {
        let subwords = sw.split();
        if subwords.len() >= 2 {
            return (i, subwords);
        }
    }
    (0, vec![])
}
