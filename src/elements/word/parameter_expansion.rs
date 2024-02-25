//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::{Subword, SubwordType};

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    for i in word.scan_pos("$") {
        replace_variable(&mut word.subwords[i..], core);
    }
    for s in &mut word.subwords {
        replace_parameter(s, core);
    }
}

fn replace_parameter(sw: &mut Box<dyn Subword>, core: &mut ShellCore) {
    if sw.get_type() == SubwordType::Parameter {
        let text = sw.get_text();
        let v = core.get_param_ref(&text[1..]);
        sw.set(SubwordType::Other, &v);
    }
}

fn replace_variable(subwords: &mut [Box<dyn Subword>], core: &mut ShellCore) {
    let mut text = String::new();
    let mut pos = 1;
    for s in &mut subwords[1..] {
        if s.get_type() != SubwordType::VarName {
            break;
        }

        text += s.get_text();
        pos += 1;
    }

   let v = core.get_param_ref(&text);
    subwords[0].set(SubwordType::Other, &v);
    subwords[1..pos].iter_mut().for_each(|s| s.clear());
}
