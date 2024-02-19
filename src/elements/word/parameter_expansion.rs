//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::SubwordType;

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    for i in word.scan_pos("$") {
        for j in i+1..word.subwords.len() {
            if word.subwords[j].get_type() != SubwordType::VarName {
                break;
            }

            let right = word.subwords[j].clone();
            word.subwords[i].merge(SubwordType::Parameter, &right);
            word.subwords[j].clear();
        }
    }
    word.subwords.iter_mut().for_each(|w| w.parameter_expansion(core));
}
