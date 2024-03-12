//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::SubwordType;
use crate::elements::word::Word;
use glob;
use glob::{GlobError, MatchOptions};
use std::path::PathBuf;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let org = word.clone();
    let ans = do_glob(&word.text)
              .into_iter()
              .map(|p| rewrite(word, &p))
              .collect::<Vec<Word>>();

    if ans.len() > 0 {
        ans
    }else{
        vec![org]
    }
}

fn do_glob(path: &str) -> Vec<String> {
    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    };

    if let Ok(ps) = glob::glob_with(&path, options) {
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
