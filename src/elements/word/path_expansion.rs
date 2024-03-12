//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::SubwordType;
use crate::elements::word::Word;
use glob;
use glob::GlobError;
use std::path::PathBuf;

fn to_string(path :&Result<PathBuf, GlobError>) -> String {
    match path {
        Ok(p) => p.to_string_lossy().to_string(),
        _ => panic!("sush: unexpected path name"),
    }
}

fn has_glob_symbol(w: &Word) -> bool {
    for sw in &w.subwords {
        let t = sw.get_text();
        if t == "[" || t == "]" || t == "*" || t == "?" {
            return true;
        }
    }

    false
}

pub fn eval(word: &mut Word, _core: &mut ShellCore) -> Vec<Word> {
    if ! has_glob_symbol(word) {
        return vec![word.clone()];
    }

    let mut ans = vec![];
    if let Ok(paths) = glob::glob(&word.text) {
        for p in paths.map(|p| to_string(&p)).collect::<Vec<String>>() {
            let mut w = word.clone();
            while w.subwords.len() > 1 {
                w.subwords.pop();
            }
            w.subwords[0].set(SubwordType::Other, &p);
            ans.push(w);
        }
    }

    if ans.len() == 0 {
        ans.push(word.clone());
    }
    ans
}
