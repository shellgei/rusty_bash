//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::exec::ExecError;
use crate::elements::word::Word;
use crate::elements::subword::Subword;
use crate::elements::subword::parameter::Parameter;

pub fn eval(word: &mut Word, core: &mut ShellCore) -> Result<(), ExecError> {
    for i in word.scan_pos("$") {
        connect_names(&mut word.subwords[i..]);
    }
    let mut tmp = vec![];
    for w in word.subwords.iter_mut() {
        let mut new_objs = w.substitute(core)?;
        match new_objs.is_empty() {
            true  => tmp.push(w.clone()),
            false => tmp.append(&mut new_objs),
        }
    }

    word.subwords = tmp;
    word.text = word.subwords.iter().map(|sw| sw.get_text().to_string()).collect::<Vec<String>>().join("");
    Ok(())
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
        let mut sw = Parameter::default();
        sw.text = text;
        subwords[0] = Box::new(sw);
        subwords[1..pos].iter_mut().for_each(|s| s.set_text(""));
    }
}
