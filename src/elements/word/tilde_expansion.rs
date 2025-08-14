//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::error::exec::ExecError;
use crate::ShellCore;
//use crate::elements::subword::Subword;
use super::WordMode;
use nix::unistd::User;

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    if word.subwords.len() > 1
        && word.subwords[1..].iter().any(|sw| sw.get_text() == "=")
        && !core.options.query("posix") {
        return eval_multi(word, core);
    }
    if let Some(WordMode::RightOfSubstitution) = word.mode {
        return eval_multi(word, core);
    }

    eval_single(word, core)
}

fn eval_single(word: &mut Word, core: &mut ShellCore) {
    let length = match prefix_length(word) {
        0 => return,
        n => n,
    };

    let text: String = word.subwords[1..length]
        .iter()
        .map(|e| e.get_text().to_string())
        .collect::<Vec<String>>()
        .concat();

    let value = get_value(&text, core).unwrap_or_default();
    if value.is_empty() {
        return;
    }
    word.text = value.clone();
    word.subwords[0] = From::from(&value);
    word.subwords[1..length]
        .iter_mut()
        .for_each(|w| w.set_text(""));
}

pub fn eval_multi(word: &mut Word, core: &mut ShellCore) {
    let mut ans_sws = vec![];
    let mut tmp = vec![];
    for sw in &word.subwords {
        if sw.get_text() == ":" {
            let mut w = Word::from(tmp.clone());
            eval_single(&mut w, core);
            ans_sws.append(&mut w.subwords);
            tmp.clear();
            ans_sws.push(sw.clone());
        } else {
            tmp.push(sw.clone());
        }
    }

    if !tmp.is_empty() {
        let mut w = Word::from(tmp.clone());
        eval_single(&mut w, core);
        ans_sws.append(&mut w.subwords);
    }

    word.subwords = ans_sws;
    word.text = word
        .subwords
        .iter()
        .map(|e| e.get_text().to_string())
        .collect::<Vec<String>>()
        .concat();
}

fn prefix_length(word: &Word) -> usize {
    if word.subwords.is_empty() || word.subwords[0].get_text() != "~" {
        return 0;
    }

    let len_colon = match word.subwords.iter().position(|e| e.get_text() == ":") {
        None => word.subwords.len(),
        Some(n) => n,
    };

    let len_slash = match word.subwords.iter().position(|e| e.get_text() == "/") {
        None => word.subwords.len(),
        Some(n) => n,
    };

    std::cmp::min(len_colon, len_slash)
}

fn get_value(text: &str, core: &mut ShellCore) -> Result<String, ExecError> {
    let key = match text {
        "" => "HOME",
        "+" => "PWD",
        "-" => "OLDPWD",
        _ => return Ok(get_home_dir(text)),
    };

    core.db.get_param(key)
}

fn get_home_dir(user: &str) -> String {
    match User::from_name(user) {
        Ok(Some(u)) => u.dir.into_os_string().into_string().unwrap(),
        _ => String::new(),
    }
}
