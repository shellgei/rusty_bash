//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::SubwordType;
use crate::elements::word::Word;
use glob;
use glob::{GlobError, MatchOptions};
use std::path::PathBuf;

pub fn eval(word: &Word) -> Vec<Word> {
    let paths = expand(&word.text);

    if paths.len() > 0 {
        let mut tmp = word.clone();
        paths.iter()
             .map(|p| rewrite(&mut tmp, &p))
             .collect()
    }else{
        vec![word.clone()]
    }
}

fn expand(path: &str) -> Vec<String> {
    let opts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    };

    let mut ans = match glob::glob_with(&path, opts) {
        Ok(ps) => ps.map(|p| to_str(&p))
                    .filter(|s| s != "").collect(), 
        _ => return vec![],
    };

    absorb_dialect(path, &mut ans);
    ans.sort();
    ans
}

fn to_str(path :&Result<PathBuf, GlobError>) -> String {
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

fn absorb_dialect(org: &str, paths: &mut Vec<String>) {
    if let Some(tail1) = org.chars().last() {
        if tail1 == '/' {
            paths.iter_mut().for_each(|p| add_slash(p));
        }
    }

    if ! org.starts_with("./") {
            paths.iter_mut().for_each(|p| remove_dot_slash(p));
    }
}

fn add_slash(path: &mut String) {
    if let Some(tail2) = path.chars().last() {
        if tail2 != '/' {
            path.push('/');
        }
    }
}

fn remove_dot_slash(path: &mut String) {
    if path.starts_with("./") && path.len() >= 3 {
        path.remove(0);
        path.remove(0);
    }
}
