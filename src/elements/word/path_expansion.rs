//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use crate::elements::word::Word;
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let paths = expand(&word.make_glob_string());
    if paths.len() == 0 {
        return vec![word.clone()];
    }

    let subwd = |path| Box::new(SimpleSubword{ text: path });
    let wd = |path| Word::from( subwd(path) as Box::<dyn Subword>);
    paths.iter().map(|p| wd(p.to_string())).collect()
}

fn expand(globstr: &str) -> Vec<String> {
    if "*?@+![".chars().all(|c| ! globstr.contains(c)) {
        return vec![];
    }

    directory::glob("", &globstr, extglob)
}
