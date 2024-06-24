//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use crate::utils::glob::compare;
use std::fs;
use std::path::Path;
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
    && globstr.find("@") == None
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

    let mut ans_cands: Vec<String> = vec![start_dir.to_string()];
    let mut tmp_ans_cands = vec![];
    for glob_elem in glob_elems {
        for cand in ans_cands {
            tmp_ans_cands.extend( expand_sub(&cand, &glob_elem) );
        }
        ans_cands = tmp_ans_cands.clone();
        tmp_ans_cands.clear();
    }

    ans_cands.iter_mut().for_each(|e| {e.pop();} );
    ans_cands.sort();
    ans_cands
}

fn expand_sub(cand: &str, glob_elem: &str) -> Vec<String> {
    let mut ans = vec![];
    if glob_elem == "" || glob_elem == "." || glob_elem == ".." {
        return vec![cand.to_string() + glob_elem + "/"];
    }

    let dir = match cand {
        "" => ".",
        x  => x, 
    };

    if ! Path::new(dir).is_dir() {
        return vec![];
    }
    let readdir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        _      => return vec![],
    };

    for e in readdir {
        let filename = match e {
            Ok(p) => p.file_name().to_string_lossy().to_string(),
            _ => continue,
        };

        if let Some(a) = comp(&filename, cand, glob_elem) {
            ans.push(a);
        }
    }

    if let Some(a) = comp(&".".to_string(), cand, glob_elem) {
        ans.push(a);
    }
    if let Some(a) = comp(&"..".to_string(), cand, glob_elem) {
        ans.push(a);
    }
    ans
}

fn comp(filename: &String, cand: &str, glob_elem: &str) -> Option<String> {
    if compare(&filename, &glob_elem) {
        if ! filename.starts_with(".") || glob_elem.starts_with(".") {
            return Some(cand.to_owned() + &filename + "/");
        }
    }

    None
}

fn rewrite(word: &mut Word, path: &str) -> Word {
    word.subwords[0] = Box::new( SimpleSubword{ text: path.to_string() } );
    while word.subwords.len() > 1 {
        word.subwords.pop();
    }
    word.clone()
}
