//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::SubwordType;

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    eprint!("parse of {}: ", &word.text);
    for sw in &word.subwords {
        if sw.get_type() == SubwordType::VarName {
            eprint!("NAME");
        }else{
            eprint!("{}", sw.get_text());
        }
    }
    eprintln!("");
    word.subwords.iter_mut().for_each(|w| w.parameter_expansion(core));
}
