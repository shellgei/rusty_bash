//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::Subword;
use crate::elements::word::Word;
type SplitResult = (usize, Vec<(Box<dyn Subword>, bool)>);

pub fn eval(word: &Word, core: &mut ShellCore) -> Vec<Word> {
    if !core.db.exist("IFS") {
        let _ = core.db.set_param("IFS", " \t\n", None);
    }

    let ifs = core.db.get_param("IFS").unwrap();

    let (pos, mut split) = match find_pos(word, &ifs) {
        Some(res) => res,
        None => return vec![word.clone()],
    };
    //if split.len() <= 0 {
    // }

    let mut left = word.subwords[..pos].to_vec();
    let remain = split[0].1;
    left.push(split.remove(0).0);
    let mut ans = vec![gen_word(left, remain)];

    while split.len() >= 2 {
        let remain = split[0].1;
        ans.push(gen_word(vec![split.remove(0).0], remain));
    }

    let remain = split[0].1;
    let mut right = gen_word(word.subwords[pos + 1..].to_vec(), remain);
    right.subwords.insert(0, split.remove(0).0);

    [ans, eval(&right, core)].concat()
}

fn gen_word(sws: Vec<Box<dyn Subword>>, remain: bool) -> Word {
    Word {
        subwords: sws,
        do_not_erase: remain,
        ..Default::default()
    }
}

pub fn find_pos(word: &Word, ifs: &str) -> Option<SplitResult> {
    let mut strip_left = true;
    for (i, sw) in word.subwords.iter().enumerate() {
        if let Some(split) = sw.split(ifs, strip_left)
            && split.len() >= 2
        {
            return Some((i, split));
        }

        if !sw.get_text().is_empty() {
            let prev_char = sw.get_text().chars().last();
            strip_left = prev_char.is_none() || " \t\n".contains(prev_char.unwrap());
        }
    }
    None
}
