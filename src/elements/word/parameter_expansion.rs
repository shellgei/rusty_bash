//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::{Subword, SubwordType};

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    for i in word.scan_pos("$") {
        connect_names(&mut word.subwords[i..]);
    }
    word.subwords
        .iter_mut()
        .for_each(|w| w.parameter_expansion(core));
}

pub fn connect_names(subwords: &mut [Box<dyn Subword>]) {
    for i in 1..subwords.len() {
        if subwords[i].get_type() != SubwordType::VarName {
            return;
        }

        let right = subwords[i].clone();
        subwords[0].merge(SubwordType::Parameter, &right);
        subwords[i].clear();
    }
}
