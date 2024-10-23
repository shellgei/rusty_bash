//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;

pub fn eval(word: &mut Word) -> Vec<Word> {
    vec![word.clone()]
}
