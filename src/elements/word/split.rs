//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::{Subword, SubwordType};

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    for (i, sw) in word.subwords.iter().enumerate() {
        if sw.get_type() == SubwordType::SingleQuoted 
        || sw.get_type() == SubwordType::DoubleQuoted {
            continue;
        }
        let split = sw.split(core);
        if split.len() == 1 {
            continue;
        }

        let mut ans = rearrange(word, split, i);
        let last = ans.pop().unwrap();
        ans.append(&mut eval(&last, core));
        return ans;
    }

    vec![word.clone()]
}

fn rearrange(word: &Word, subwords: Vec<Box<dyn Subword>>, pos: usize) -> Vec<Word> {
    let mut ans = vec![];
    let split_len = subwords.len();

    let mut left = Word::new();
    left.subwords = word.subwords[..pos].to_vec();
    left.subwords.push(subwords[0].clone());
    ans.push(left);

    for sw in subwords[1..split_len-1].iter() {
        let mut mid = Word::new();
        mid.subwords = vec![sw.clone()];
        ans.push(mid);
    }

    let mut right = Word::new();
    right.subwords = vec![subwords[split_len-1].clone()];
    right.subwords.append(&mut word.subwords[pos+1..].to_vec());
    ans.push(right);

    ans
}
