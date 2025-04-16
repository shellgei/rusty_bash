//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::exit;
use super::extglob;
use super::{GlobElem, MetaChar};

pub fn shave_word(word: &String, pattern: &Vec<GlobElem>) -> Vec<String> {
    dbg!("{:?}", &pattern);
    let mut candidates = vec![word.to_string()];
    pattern.iter().for_each(|w| shave(&mut candidates, &w) );
    candidates
}

pub fn shave(candidates: &mut Vec<String>, w: &GlobElem) {
    match w {
        GlobElem::Normal(s) => normal(candidates, &s),
        GlobElem::Symbol('?') => question(candidates),
        GlobElem::Symbol('*') => asterisk(candidates),
        GlobElem::OneOf(not, cs) => one_of(candidates, &cs, *not),
        GlobElem::ExtGlob(prefix, ps) => extglob::shave(candidates, *prefix, &ps),
        GlobElem::Symbol(_) => exit::internal("Unknown glob symbol"),
    }
}

fn normal(cands: &mut Vec<String>, s: &String) {
    cands.retain(|c| c.starts_with(s) );
    cands.iter_mut().for_each(|c| {*c = c.split_off(s.len());});
}

fn question(cands: &mut Vec<String>) {
    cands.retain(|c| c.len() != 0 );
    let len = |c: &String| c.chars().nth(0).unwrap().len_utf8();
    cands.iter_mut().for_each(|c| {*c = c.split_off(len(c));});
}

fn asterisk(cands: &mut Vec<String>) {
    if cands.is_empty() {
        return;
    }

    let mut ans = vec!["".to_string()];
    for cand in cands.iter_mut() {
        let mut len = 0;
        for c in cand.chars() {
            ans.push(cand.clone().split_off(len));
            len += c.len_utf8();
        }
    }
    *cands = ans;
}

fn one_of(cands: &mut Vec<String>, cs: &Vec<MetaChar>, not_inv: bool) {
    cands.retain(|cand| cand.len() != 0 );
    cands.retain(|cand| cs.iter().any(|c| compare_head(cand, c)) == not_inv );
    let len = |c: &String| c.chars().nth(0).unwrap().len_utf8();
    cands.iter_mut().for_each(|c| {*c = c.split_off(len(c));});
}

fn compare_head(cand: &String, c: &MetaChar) -> bool {
    let head = cand.chars().nth(0).unwrap();
    match c {
        MetaChar::Normal(c) => head == *c,
        MetaChar::Range(f, t) => range_check(*f, *t, head),
        MetaChar::CharClass(cls) => charclass_check(&cls, head),
    }
}

fn range_check(from: char, to: char, c: char) -> bool {
    if ('0' <= from && from <= to && to <= '9')
    || ('a' <= from && from <= to && to <= 'z')
    || ('A' <= from && from <= to && to <= 'Z') {
        return from <= c && c <= to;
    }
    false
}

fn charclass_check(cls: &str, c: char) -> bool {
    match cls { //TODO: rough implementation, no test except for [:space:]
        "[:alnum:]" => c.is_ascii_alphanumeric(),
        "[:alpha:]" => c.is_ascii_alphabetic(),
        "[:ascii:]" => c.is_ascii(),
        "[:blank:]" => c == ' ' || c == '\t',
        "[:cntrl:]" => c.is_ascii_control(),
        "[:digit:]" => c.is_digit(10),
        "[:graph:]" => c.is_ascii_graphic(),
        "[:lower:]" => c.is_ascii_lowercase(),
        "[:print:]" => c.is_ascii() && ! c.is_ascii_control(),
        "[:punct:]" => c.is_ascii_punctuation(),
        "[:space:]" => c.is_ascii_whitespace(),
        "[:upper:]" => c.is_ascii_uppercase(),
        "[:word:]" => c.is_ascii_alphanumeric() || c == '_',
        "[:xdigit:]" => c.is_digit(16),
        _ => false,
    }
}
