//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::Subword;
use crate::elements::subword::single_quoted::SingleQuoted;

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    if ! core.db.has_value("IFS") {
        let _ = core.db.set_param("IFS", " \t\n", None);
    }else if core.db.get_param("IFS").unwrap() == "" {
        return vec![word.clone()];
    }

    let ifs = core.db.get_param("IFS").unwrap();

    let (pos, mut split) = find_pos(word, &ifs);
    if split.is_empty() {
        return vec![word.clone()];
    }

    //dbg!("{:?}", &split);

    let gen_word = |sws, remain| Word{
        text: String::new(),
        subwords: sws,
        do_not_erase: remain };

    let mut left = word.subwords[..pos].to_vec();
    let remain = split[0].1;
    left.push(split.remove(0).0);
    let mut ans = vec![gen_word(left, remain)];

    while split.len() >= 2 {
        let remain = split[0].1;
        ans.push(gen_word(vec![split.remove(0).0], remain));
    }

    let remain = split[0].1;
    //dbg!("{:?}", &remain);
    let mut right = gen_word(word.subwords[pos+1..].to_vec(), remain);
    right.subwords.insert(0, split.remove(0).0);
    if remain {
        right.subwords.insert(0, SingleQuoted{ text: "''".to_string() }.boxed_clone());
    }

    [ ans, eval(&right, core) ].concat()
}

pub fn find_pos(word: &Word, ifs: &str) -> (usize, Vec<(Box<dyn Subword>, bool)>) {
    for (i, sw) in word.subwords.iter().enumerate() {
        let split = sw.split(ifs);
        if split.len() >= 2 {
            return (i, split);
        }
    }
    (0, vec![])
}
