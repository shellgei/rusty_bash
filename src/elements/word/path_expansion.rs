//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::utils::directory;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let paths = expand(&word.make_glob_string());
    if paths.len() == 0 {
        return vec![word.clone()];
    }

    paths.iter().map(|p| Word::from(p.as_str())).collect()
}

fn expand(pattern: &str) -> Vec<String> {
    if "*?+![".chars().all(|c| ! pattern.contains(c)) {
        return vec![];
    }

    let div = pattern.split("/");
    let last = div.last().unwrap();
    let dir = &pattern[0..pattern.len()-last.len()];
    directory::glob(dir, last)
}

