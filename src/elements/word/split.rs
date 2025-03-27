//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::Subword;

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    if ! core.db.has_value("IFS") {
        let _ = core.db.set_param("IFS", " \r\n", None);
    }else if core.db.get_param("IFS").unwrap() == "" {
        return vec![word.clone()];
    }

    let ifs = core.db.get_param("IFS").unwrap();
    let (pos, mut split) = find_pos(word, &ifs);
    if split.is_empty() {
        return vec![word.clone()];
    }

    let gen_word = |sws| Word{
        text: String::new(),
        subwords: sws,
        do_not_erase: ! ifs.contains(" ") };

    let mut left = gen_word(word.subwords[..pos].to_vec());
    left.subwords.push(split.remove(0));

    let mut ans = vec![left];
    while split.len() >= 2 {
        ans.push(gen_word(vec![split.remove(0)]));
    }

    let mut right = gen_word(word.subwords[pos+1..].to_vec());
    right.subwords.insert(0, split.remove(0));

    [ ans, eval(&right, core) ].concat()
}

pub fn find_pos(word: &Word, ifs: &str) -> (usize, Vec<Box<dyn Subword>>) {
    for (i, sw) in word.subwords.iter().enumerate() {
        let split = sw.split(ifs);
        if split.len() >= 2 {
            return (i, split);
        }
    }
    (0, vec![])
}
