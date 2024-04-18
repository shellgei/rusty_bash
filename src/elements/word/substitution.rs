//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::{Subword, SubwordType};

pub fn eval(word: &mut Word, core: &mut ShellCore) -> bool {
    for i in word.scan_pos("$") {
        connect_names(&mut word.subwords[i..]);
    }
    word.subwords.iter_mut().all(|w| w.substitute(core))
}

fn connect_names(subwords: &mut [Box<dyn Subword>]) {
    let mut text = "$".to_string();
    let mut pos = 1;
    for s in &mut subwords[1..] {
        if s.get_type() != SubwordType::VarName {
            break;
        }
        text += s.get_text();
        pos += 1;
    }

    if pos > 1 {
        subwords[0].set(SubwordType::Parameter, &text);
        subwords[1..pos].iter_mut().for_each(|s| s.clear());
    }
}
