//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::utils::glob;
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let paths = expand(&word.make_glob_string());

    if paths.len() > 0 {
        let mut tmp = word.clone();
        paths.iter()
             .map(|p| rewrite(&mut tmp, &p))
             .collect()
    }else{
        vec![word.clone()]
    }
}

fn expand(globstr: &str) -> Vec<String> {
    if globstr.find("*") == None 
    && globstr.find("?") == None
    && globstr.find("@") == None
    && globstr.find("[") == None {
        return vec![];
    }
        
    let mut ans_cands = vec!["".to_string()];
    let mut tmp_ans_cands = vec![];

    for glob_elem in globstr.split("/") {
        for cand in ans_cands {
            tmp_ans_cands.extend( glob::glob_in_dir(&cand, &glob_elem) );
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
