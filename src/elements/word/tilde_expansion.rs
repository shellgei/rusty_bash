//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    let length = match prefix_length(word) {
        0 => return,
        n => n,
    };

    let text: String = word.subwords[1..length].iter()
               .map(|e| e.get_text().to_string())
               .collect::<Vec<String>>()
               .concat();

    eprintln!("PREFIX: {}", text);
}

fn prefix_length(word: &Word) -> usize {
    if word.subwords.len() == 0 || word.subwords[0].get_text() != "~" {
        return 0;
    }

    match word.subwords.iter().position(|e| e.get_text() == "/") {
        None    => word.subwords.len(),
        Some(n) => n,
    }
}
