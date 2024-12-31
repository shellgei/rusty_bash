//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use crate::elements::word::Word;
use crate::utils::directory;
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word, extglob: bool) -> Vec<Word> {
    let paths = expand(&word.make_glob_string(), extglob);
    if paths.is_empty() {
        return vec![word.clone()];
    }

    let subwd = |path| Box::new(SimpleSubword{ text: path });
    let wd = |path| Word::from( subwd(path) as Box::<dyn Subword>);
    paths.iter().map(|p| wd(p.to_string())).collect()
}

fn expand(pattern: &str, extglob: bool) -> Vec<String> {
    if "*?@+![".chars().all(|c| ! pattern.contains(c)) {
        return vec![];
    }
        
    let mut paths = vec!["".to_string()];

    for dir_glob in pattern.split("/") {
        paths = paths.iter()
                .map(|c| directory::glob(&c, &dir_glob, extglob) )
                .collect::<Vec<Vec<String>>>()
                .concat();
    }

    paths.iter_mut().for_each(|e| {e.pop();} );
    paths.sort();
    paths
}
