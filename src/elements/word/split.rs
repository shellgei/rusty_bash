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

    let left = Word::concat_new(&[ &word.subwords[..pos], &[subws.remove(0)] ]);
    let mut words = vec![left];
    while subws.len() > 1 {
        words.push(Word::new(&[subws.remove(0)]));
    }
    let right = Word::concat_new(&[ &[subws.remove(0)], &word.subwords[pos+1..]]);

    [ words, eval(&right, core) ].concat()
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
