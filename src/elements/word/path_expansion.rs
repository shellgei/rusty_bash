//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::utils::directory;
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word, extglob: bool) -> Vec<Word> {
    let paths = expand(&word.make_glob_string(), extglob);
    if paths.len() == 0 {
        return vec![word.clone()];
    }

    let mut tmp = word.clone();
    paths.iter().map(|p| rewrite(&mut tmp, &p)).collect()
}

fn expand(globstr: &str, extglob: bool) -> Vec<String> {
    if "*?@+![".chars().all(|c| ! globstr.contains(c)) {
        return vec![];
    }
        
    let mut ans_cands = vec!["".to_string()];
    let mut tmp_ans_cands = vec![];

    for glob_elem in globstr.split("/") {
        for cand in ans_cands {
            tmp_ans_cands.extend( directory::glob(&cand, &glob_elem, extglob) );
        }
        ans_cands = tmp_ans_cands.clone();
        tmp_ans_cands.clear();
    }

    ans_cands.iter_mut().for_each(|e| {e.pop();} );
    ans_cands.sort();
    ans_cands
}

fn rewrite(word: &mut Word, path: &str) -> Word {
    word.subwords[0] = Box::new( SimpleSubword{ text: path.to_string() } );
    while word.subwords.len() > 1 {
        word.subwords.pop();
    }
    word.clone()
}
