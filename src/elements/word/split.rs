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

        let gen_word = |sws| Word{ text: "".to_string(), subwords: sws};

        let mut ans = vec![];
        for sw in &split[1..split.len()-1] {
            ans.push(gen_word(vec![sw.clone()]));
        }

        let mut left = gen_word(word.subwords[..i].to_vec());
        left.push(&split[0].clone());
        ans.insert(0, left);

        let mut right = gen_word(vec![split.pop().unwrap()]);
        right.subwords.append(&mut word.subwords[i+1..].to_vec());

        ans.append(&mut eval(&right, core));
        return ans;
    }

    vec![word.clone()]
}
