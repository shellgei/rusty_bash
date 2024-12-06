//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::exit;
use super::extglob;
use super::GlobElem;

pub fn shave_word(word: &String, pattern: &Vec<GlobElem>) -> Vec<String> {
    let mut candidates = vec![word.to_string()];
    pattern.iter().for_each(|w| shave(&mut candidates, &w) );
    candidates
}

pub fn shave(candidates: &mut Vec<String>, w: &GlobElem) {
    match w {
        GlobElem::Normal(s) => nonspecial(candidates, &s),
        GlobElem::Symbol('?') => question(candidates),
        GlobElem::Symbol('*') => asterisk(candidates),
        GlobElem::OneOf(not, cs) => one_of(candidates, &cs, *not),
        GlobElem::ExtGlob(prefix, ps) => extglob::shave(candidates, *prefix, &ps),
        GlobElem::Symbol(_) => exit::internal("Unknown glob symbol"),
    }
}

fn nonspecial(cands: &mut Vec<String>, s: &String) {
    cands.retain(|c| c.starts_with(s) );
    cands.iter_mut().for_each(|c| {*c = c.split_off(s.len());});
}

fn question(cands: &mut Vec<String>) {
    cands.retain(|c| c.len() != 0 );
    let len = |c: &String| c.chars().nth(0).unwrap().len_utf8();
    cands.iter_mut().for_each(|c| {*c = c.split_off(len(c));});
}

fn asterisk(cands: &mut Vec<String>) {
    if cands.len() == 0 {
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

fn one_of(cands: &mut Vec<String>, cs: &Vec<char>, not_inv: bool) {
    cands.retain(|cand| cs.iter().any(|c| cand.starts_with(*c)) == not_inv );
    let len = |c: &String| c.chars().nth(0).unwrap().len_utf8();
    cands.iter_mut().for_each(|c| {*c = c.split_off(len(c));});
}
