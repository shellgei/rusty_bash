//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use crate::elements::word::Word;
use crate::utils::directory;
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word, extglob: bool, nullglob: bool) -> Vec<Word> {
    let globstr = word.make_glob_string();
    if no_glob_symbol(&globstr) {
        return vec![word.clone()];
    }

    let paths = expand(&globstr, extglob);
    if paths.is_empty() {
        if nullglob {
            return vec![Word::from(&String::new())];
        }
        return vec![word.clone()];
    }

    let subwd = |path| Box::new(SimpleSubword{ text: path });
    let wd = |path| Word::from( subwd(path) as Box::<dyn Subword>);
    paths.iter().map(|p| wd(p.to_string())).collect()
}

fn no_glob_symbol(pattern: &str) -> bool {
    "*?@+![".chars().all(|c| ! pattern.contains(c))
}

pub fn expand(pattern: &str, extglob: bool) -> Vec<String> {
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
