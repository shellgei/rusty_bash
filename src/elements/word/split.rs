//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::Subword;

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    for (i, sw) in word.subwords.iter().enumerate() {
        let split = sw.split();
        if split.len() <= 1 {
            continue;
        }

        let mut ans = rearrange(word, split, i);
        let last = ans.pop().unwrap();
        ans.append(&mut eval(&last, core));
        return ans;
    }

    vec![word.clone()]
}

fn rearrange(word: &Word, _: Vec<Box<dyn Subword>>, _: usize) -> Vec<Word> {
    vec![word.clone()]
}
