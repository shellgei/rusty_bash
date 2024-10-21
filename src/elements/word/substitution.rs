//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::Subword;
use crate::elements::subword::parameter::Parameter;

pub fn eval(word: &mut Word, core: &mut ShellCore) -> bool {
    for i in word.scan_pos("$") {
        connect_names(&mut word.subwords[i..]);
    }
    if ! word.subwords.iter_mut().all(|w| w.substitute(core)) {
        return false;
    }
    alternative_replace(word);
    true
}

fn alternative_replace(word: &mut Word) {
    let mut pos = 0;
    while pos < word.subwords.len() {
        let sws = word.subwords[pos].get_alternative_subwords();
        if sws.len() == 0 {
            pos += 1;
            continue;
        }

        word.subwords.remove(pos);
        for s in sws {
            word.subwords.insert(pos, s.clone());
            pos += 1;
        }
    }
}

fn connect_names(subwords: &mut [Box<dyn Subword>]) {
    let mut text = "$".to_string();
    let mut pos = 1;
    for s in &mut subwords[1..] {
        if ! s.is_name() {
            break;
        }
        text += s.get_text();
        pos += 1;
    }

    if pos > 1 {
        subwords[0] = Box::new(Parameter{ text: text });
        subwords[1..pos].iter_mut().for_each(|s| s.set_text(""));
    }
}
