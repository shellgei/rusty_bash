//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::utils::directory;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let paths = expand(&word.make_glob_string());
    if paths.is_empty() {
        return vec![word.clone()];
    }

    paths.iter().map(|p| Word::from(p.as_str())).collect()
}

fn expand(pattern: &str) -> Vec<String> {
    if "*?+![".chars().all(|c| ! pattern.contains(c)) {
        return vec![];
    }

    let mut paths = vec!["".to_string()];
 
    for dir_pat in pattern.split("/") {
        paths = paths.iter()
                .map(|c| directory::glob(c, dir_pat) )
                .collect::<Vec<Vec<String>>>()
                .concat();
    }

    paths.iter_mut().for_each(|e| {e.pop();} );
    paths.sort();
    paths
}
