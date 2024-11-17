//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use crate::elements::word::Word;
use crate::utils::directory;
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word, extglob: bool) -> Vec<Word> {
    let paths = expand(&word.make_glob_string(), extglob);
    if paths.len() == 0 {
        return vec![word.clone()];
    }

    let subwd = |path| Box::new(SimpleSubword{ text: path });
    let wd = |path| Word::from( subwd(path) as Box::<dyn Subword>);
    paths.iter().map(|p| wd(p.to_string())).collect()
}

fn expand(globstr: &str, extglob: bool) -> Vec<String> {
    if "*?@+![".chars().all(|c| ! globstr.contains(c)) {
        return vec![];
    }
        
    let mut ans_cands = vec!["".to_string()];

    for dir_glob in globstr.split("/") {
        ans_cands = ans_cands.iter()
                    .map(|c| directory::glob(&c, &dir_glob, extglob) )
                    .collect::<Vec<Vec<String>>>()
                    .concat();
    }

    ans_cands.iter_mut().for_each(|e| {e.pop();} );
    ans_cands.sort();
    ans_cands
}
