//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    let (pos, mut words) = split(word);
    if words.len() == 0 {
        return vec![word.clone()];
    }

    words[0].subwords = vec![
                            word.subwords[..pos].to_vec(),
                            words[0].subwords.clone()
                        ].concat();

    let mut right = words.pop().unwrap();
    right.subwords.append(&mut word.subwords[pos+1..].to_vec());
    
    [ words, eval(&right, core) ].concat()
}

pub fn split(word: &Word) -> (usize, Vec<Word>) {
    for (i, sw) in word.subwords.iter().enumerate() {
        let subwords = sw.split();
        if subwords.len() >= 2 {
            let words = subwords.iter()
                        .map(|s| Word::new(vec![s.clone()]))
                        .collect();
            return (i, words);
        }
    }
    (0, vec![])
}

