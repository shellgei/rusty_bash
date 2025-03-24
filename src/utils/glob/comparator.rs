//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::GlobElem;

pub fn shave_word(word: &String, pattern: &[GlobElem]) -> Vec<String> {
    let mut candidates = vec![word.to_string()];
    pattern.iter().for_each(|w| shave(&mut candidates, w) );
    candidates
}

pub fn shave(candidates: &mut Vec<String>, w: &GlobElem) {
    match w {
        GlobElem::Normal(s) => normal(candidates, s),
        GlobElem::Symbol('?') => question(candidates),
        GlobElem::Symbol('*') => asterisk(candidates),
        GlobElem::OneOf(not, cs) => one_of(candidates, cs, *not),
        _ => panic!("Unknown glob symbol"),
    }
}

fn normal(cands: &mut Vec<String>, pat: &String) {
    cands.retain(|c| c.starts_with(pat) );
    cands.iter_mut().for_each(|c| {*c = c.split_off(pat.len());});
}

fn question(cands: &mut Vec<String>) {
    cands.retain(|c| ! c.is_empty() );
    let len = |c: &String| c.chars().next().unwrap().len_utf8();
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

fn one_of(cands: &mut Vec<String>, cs: &[char], not_inv: bool) {
    cands.retain(|cand| cs.iter().any(|c| cand.starts_with(*c)) == not_inv );
    cands.retain(|cand| !cand.is_empty() );
    let len = |c: &String| c.chars().next().unwrap().len_utf8();
    cands.iter_mut().for_each(|c| {*c = c.split_off(len(c));});
}
