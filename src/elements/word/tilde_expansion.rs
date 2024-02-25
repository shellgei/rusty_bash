//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::SubwordType;

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    if word.subwords.len() == 0 
    || word.subwords[0].get_text() != "~" {
        return;
    }

    let mut text = String::new();
    let mut pos = 1;
    for sw in &word.subwords[1..] {
        if sw.get_text() == "/" {
            break;
        }
        text += &sw.get_text();
        pos += 1;
    }

    let v = get_value(&text, core);
    word.subwords[0].set(SubwordType::Other, &v);
    for i in 1..pos {
        word.subwords[i].clear();
    }
}

fn get_value(text: &str, core: &mut ShellCore) -> String {
    let key = match text {
        "" => "HOME",
        "+" => "PWD", 
        "-" => "OLDPWD", 
        _ => text,
    };

    core.get_param_ref(key).to_string()
}
