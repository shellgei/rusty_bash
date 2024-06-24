//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::utils::glob::compare;
use glob;
use glob::{GlobError, MatchOptions};
use regex::Regex;
use std::{env, fs};
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

fn expand(globstr: &str) -> Vec<String> {
    if globstr.find("*") == None 
    && globstr.find("?") == None
    && globstr.find("[") == None {
        return vec![];
    }
        
    let mut glob_elems: Vec<String> = globstr.split("/").map(|s| s.to_string()).collect();
    let start_dir = match globstr.starts_with("/") {
        true  => {
            glob_elems.remove(0);
            "/"
        },
        false => "",
    };
    //dbg!("{:?}", &dirs);

    let mut ans_cands: Vec<String> = vec![start_dir.to_string()];
    let mut tmp_ans_cands = vec![];
    for glob_elem in glob_elems {
        for mut cand in ans_cands {
            tmp_ans_cands.extend( expand_sub(&cand, &glob_elem) );
        }
        ans_cands = tmp_ans_cands.clone();
        tmp_ans_cands.clear();
    }

    ans_cands.iter_mut().for_each(|e| {e.pop();} );
    eprintln!("{:?}", &ans_cands);
    ans_cands
}

fn expand_sub(cand: &str, glob_elem: &str) -> Vec<String> {
    dbg!("{:?} {:?}", cand, glob_elem);
    let mut ans: Vec<String> = vec![];

    if glob_elem == "." || glob_elem == ".." {
        return vec![cand.to_string() + glob_elem + "/"];
    }

    let dir = match cand {
        "" => ".",
        x  => x, 
    }.to_string();

    for e in fs::read_dir(dir).unwrap() {
        let filename = match e {
            Ok(p) => p.file_name().to_string_lossy().to_string(),
            _ => continue,
        };
        eprintln!("{:?}", &filename);
        match compare(&filename, &glob_elem) {
            true  => ans.push(cand.clone().to_owned() + &filename + "/"),
            false => {},
        }
    }

    dbg!("{:?}", &ans);
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
