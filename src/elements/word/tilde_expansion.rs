//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use crate::elements::subword::SubwordType;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
        _ => return solve_home_dir(text),
    };

    core.get_param_ref(key).to_string()
}

fn solve_home_dir(user: &str) -> String {
    let reader = match File::open("/etc/passwd") {
        Ok(f) => BufReader::new(f),
        _ => return String::new(),
    };

    for line in reader.lines() {
        if let Ok(ref ln) = line {
            let split: Vec<&str> = ln.split(":").collect();
            if split.len() > 5 && user == split[0] {
                return split[5].to_string();
            }
        }
    }

    String::new()
}
