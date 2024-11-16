//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let paths = expand(&word.make_glob_string());
    vec![word.clone()]
}

fn expand(globstr: &str) -> Vec<String> {
    dbg!("{:?}", &globstr);
    vec![]
}
