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

fn expand(globstr: &str) -> Vec<String> {
    if "*?@+![".chars().all(|c| ! globstr.contains(c)) {
        return vec![];
    }

    let div = globstr.split("/");
    let last = div.last().unwrap();
    directory::files(&globstr[0..globstr.len()-last.len()])
}
