//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::utils::glob::compare;
use glob;
use glob::{GlobError, MatchOptions};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let paths = expand(&word.make_glob_string());

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
    let mut dir = match Path::new(path).parent() {
        Some(p) => p, 
        None    => return vec![],
    };

    let show_hidden = path.starts_with(&(dir.to_string_lossy().to_string() + "."));

    let mut remove_dot_slash = false;
    if dir.to_string_lossy() == "" {
        remove_dot_slash = true;
        dir = Path::new("./");
    }

    if ! dir.is_dir() {
        return vec![];
    }

    let mut ans = vec![];
    for e in fs::read_dir(dir).unwrap() {
        let p = match e {
            Ok(p) => p.path(),
            _ => continue,
        };
        let filename = p.file_name().expect("!").to_string_lossy();
        let mut cand = p.clone().into_os_string().into_string().unwrap();
        if remove_dot_slash {
            cand = cand.replacen("./", "", 1);
        }

        if ! show_hidden && filename.starts_with(".") {
            continue;
        }

        match compare(&cand, &path) {
            true  => ans.push(cand),
            false => {
                if p.is_dir() {
                    let with_slash = cand.clone() + "/";
                    match compare(&with_slash, &path) {
                        true  => ans.push(with_slash),
                        false => {},
                    }
                }
            },
        }
    }

    if path == ".*" {
        ans.push(".".to_string());
        ans.push("..".to_string());
    }
    if path == "..*" {
        ans.push("..".to_string());
    }

    ans
}

/*
fn expand(path: String) -> Vec<String> {
    let opts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    };

    let re = Regex::new(r"\*+").unwrap(); //prohibit globstar
    let fix_path = re.replace_all(&path, "*");

    let mut ans = match glob::glob_with(&fix_path, opts) {
        Ok(ps) => ps.map(|p| to_str(&p))
                    .filter(|s| s != "").collect(), 
        _ => return vec![],
    };

    absorb_dialect(&path, &mut ans);
    ans.sort();
    ans
}
*/

fn to_str(path :&Result<PathBuf, GlobError>) -> String {
    match path {
        Ok(p) => p.to_string_lossy().to_string(),
        _ => "".to_string(),
    }
}

fn rewrite(word: &mut Word, path: &str) -> Word {
    word.subwords[0] = Box::new( SimpleSubword{ text: path.to_string() } );
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

    if org.starts_with("./") {
        paths.iter_mut().for_each(|p| add_dot_slash(p));
    }else{
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

fn add_dot_slash(path: &mut String) {
    if ! path.starts_with("./") {
        path.insert(0, '/');
        path.insert(0, '.');
    }
}

fn remove_dot_slash(path: &mut String) {
    if path.starts_with("./") && path.len() >= 3 {
        path.remove(0);
        path.remove(0);
    }
}
