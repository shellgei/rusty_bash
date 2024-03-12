//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::SubwordType;
use crate::elements::word::Word;
use glob;
use glob::GlobError;
use std::path::PathBuf;

pub fn eval(word: &mut Word, _core: &mut ShellCore) -> Vec<Word> {
    let org = word.clone();
    if ! has_glob_symbol(word) {
        return vec![org];
    }

    let ans = do_glob(&word.text)
              .into_iter()
              .map(|p| rewrite(word, &p))
              .collect::<Vec<Word>>();

    if ans.len() > 0 {
        ans
    }else{
        eprintln!("NO");
        vec![org]
    }
}

fn has_glob_symbol(w: &Word) -> bool {
    for sw in &w.subwords {
        match sw.get_text() {
            "[" | "]" | "*" | "?" => return true,
            _ => continue,
        }
    }

    false
}

fn do_glob(path: &str) -> Vec<String> {
    if let Ok(ps) = glob::glob(&path) {
        ps.map(|p| to_string(&p))
          .filter(|s| s != "")
          .collect::<Vec<String>>()
    }else{
        vec![]
    }
}

fn to_string(path :&Result<PathBuf, GlobError>) -> String {
    match path {
        Ok(p) => p.to_string_lossy().to_string(),
        _ => "".to_string(),
    }
}

fn rewrite(word: &mut Word, path: &str) -> Word {
    word.subwords[0].set(SubwordType::Other, &path);
    while word.subwords.len() > 1 {
        word.subwords.pop();
    }
    word.clone()
}
