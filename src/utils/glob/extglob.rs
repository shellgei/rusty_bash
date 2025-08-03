//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::comparator;
use super::parser;
use super::GlobElem;
use crate::exit;

pub fn shave(cands: &mut Vec<String>, prefix: char, patterns: &Vec<String>) {
    match prefix {
        '?' => question(cands, patterns),
        '*' => zero_or_more(cands, patterns),
        '+' => more_than_zero(cands, patterns),
        '@' => once(cands, patterns),
        '!' => not(cands, patterns),
        _ => exit::internal("unknown extglob prefix"),
    }
}

fn question(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = cands.clone();
    for p in patterns {
        let mut tmp = cands.clone();
        parser::parse(p, true)
            .iter()
            .for_each(|w| comparator::shave(&mut tmp, w));
        ans.append(&mut tmp);
    }
    *cands = ans;
}

fn zero_or_more(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = vec![];
    let mut tmp = cands.clone();
    let mut len = tmp.len();

    while len > 0 {
        ans.extend(tmp.clone());
        once(&mut tmp, patterns);
        for a in &ans {
            tmp.retain(|t| a.as_str() != t.as_str());
        }

        len = tmp.len();
    }
    *cands = ans;
}

fn more_than_zero(cands: &mut Vec<String>, patterns: &Vec<String>) {
    //TODO: buggy
    let mut ans: Vec<String> = vec![];
    let mut tmp: Vec<String> = cands.clone();
    let mut len = tmp.len();

    while len > 0 {
        once(&mut tmp, patterns);

        for a in &ans {
            tmp.retain(|t| a.as_str() != t.as_str());
        }
        ans.extend(tmp.clone());
        len = tmp.len();
    }
    *cands = ans;
}

fn once(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = vec![];
    for p in patterns {
        let mut tmp = cands.clone();
        parser::parse(p, true)
            .iter()
            .for_each(|w| comparator::shave(&mut tmp, w));
        ans.append(&mut tmp);
    }
    *cands = ans;
}

fn not(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = vec![];
    for cand in cands.iter_mut() {
        for prefix in make_prefix_strings(cand) {
            if !once_exact_match(&prefix, patterns) {
                ans.push(cand[prefix.len()..].to_string());
            }
        }
    }
    *cands = ans;
}

fn once_exact_match(cand: &str, patterns: &Vec<String>) -> bool {
    let mut tmp = vec![cand.to_string()];
    once(&mut tmp, patterns);
    tmp.iter().any(|t| t.is_empty())
}

pub fn scan(remaining: &str) -> (usize, Option<GlobElem>) {
    let prefix = match remaining.chars().next() {
        Some(c) => c,
        None => return (0, None),
    };

    if "?*+@!".find(prefix).is_none() || remaining.chars().nth(1) != Some('(') {
        return (0, None);
    }

    let mut chars = vec![];
    let mut len = 2;
    let mut escaped = false;
    let mut nest = 0;
    let mut next_nest = false;
    let mut patterns = vec![];

    for c in remaining[len..].chars() {
        len += c.len_utf8();

        if escaped {
            chars.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }

        if c == '|' && nest == 0 {
            patterns.push(chars.iter().collect());
            chars.clear();
            continue;
        }

        if next_nest && c == '(' {
            nest += 1;
        }

        next_nest = "?*+@!".find(c).is_some();

        if c == ')' {
            match nest {
                0 => {
                    return {
                        patterns.push(chars.iter().collect());
                        (len, Some(GlobElem::ExtGlob(prefix, patterns)))
                    }
                }
                _ => nest -= 1,
            }
        }

        chars.push(c);
    }

    (0, None)
}

fn make_prefix_strings(s: &str) -> Vec<String> {
    let mut ans = vec![];
    let mut prefix = s.to_string();

    ans.push(prefix.clone());
    while !prefix.is_empty() {
        prefix.pop();
        ans.push(prefix.clone());
    }
    ans
}
