//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    for (i, sw) in word.subwords.iter().enumerate() {
        let mut split = sw.split();
        if split.len() <= 1 {
            continue;
        }

        let gen_word = |sws| Word{ text: String::new(), subwords: sws};

        let mut left = gen_word(word.subwords[..i].to_vec());
        left.subwords.push(split.remove(0));

        let mut ans = vec![left];
        while split.len() >= 2 {
            ans.push(gen_word(vec![split.remove(0)]));
        }

        let mut right = gen_word(word.subwords[i+1..].to_vec());
        right.subwords.insert(0, split.remove(0));

        ans.append(&mut eval(&right, core));
        return ans;
    }

    vec![word.clone()]
}
