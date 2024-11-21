//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod extglob;
mod parser;

#[derive(Debug)]
enum Wildcard {
    Normal(String),
    Asterisk,
    Question,
    OneOf(Vec<char>),
    NotOneOf(Vec<char>),
    ExtGlob(char, Vec<String>),
}

pub fn compare(word: &String, pattern: &str, extglob: bool) -> bool {
    get_shaved_candidates(word, pattern, extglob).iter().any(|c| c == "")
}

pub fn longest_match_length(word: &String, pattern: &str, extglob: bool) -> usize {
    word.len() - get_shaved_candidates(word, pattern, extglob).iter()
                 .map(|c| c.len()).min().unwrap_or(word.len())
}

pub fn shortest_match_length(word: &String, pattern: &str, extglob: bool) -> usize {
    word.len() - get_shaved_candidates(word, pattern, extglob).iter()
                 .map(|c| c.len()).max().unwrap_or(word.len())
}

fn get_shaved_candidates(word: &String, pattern: &str, extglob: bool) -> Vec<String> {
    let mut candidates = vec![word.to_string()];
    parser::parse(pattern, extglob).iter()
        .for_each(|w| shave(&mut candidates, &w) );

    candidates
}

fn shave(candidates: &mut Vec<String>, w: &Wildcard) {
    match w {
        Wildcard::Normal(s) => nonspecial(candidates, &s),
        Wildcard::Asterisk  => asterisk(candidates),
        Wildcard::Question  => question(candidates),
        Wildcard::OneOf(cs) => one_of(candidates, &cs, false),
        Wildcard::NotOneOf(cs) => one_of(candidates, &cs, true),
        Wildcard::ExtGlob(prefix, ps) => extglob::shave(candidates, *prefix, &ps),
    }
}

fn nonspecial(cands: &mut Vec<String>, s: &String) {
    let mut ans = vec![];

    for c in cands.into_iter() {
        if ! c.starts_with(s) {
            continue;
        }
        
        ans.push(c[s.len()..].to_string());
    }

    *cands = ans;
}

fn asterisk(cands: &mut Vec<String>) {
    let mut ans = vec![];
    for cand in cands.into_iter() {
        let mut s = String::new();
        ans.push(s.clone());
        for c in cand.chars().rev() {
            s = c.to_string() + &s.clone();
            ans.push(s.clone());
        }
    }

    *cands = ans;
}

fn question(cands: &mut Vec<String>) {
    let mut ans = vec![];
    for cand in cands.into_iter() {
        if let Some(c) = cand.chars().nth(0) {
            let len = c.len_utf8();
            ans.push(cand[len..].to_string());
        }
    }
    *cands = ans;
}

fn one_of(cands: &mut Vec<String>, cs: &Vec<char>, inverse: bool) {
    let mut ans = vec![];
    for cand in cands.into_iter() {
        if cs.iter().any(|c| cand.starts_with(*c)) ^ inverse {
            let h = cand.chars().nth(0).unwrap();
            ans.push(cand[h.len_utf8()..].to_string());
        }
    }
    *cands = ans;
}
