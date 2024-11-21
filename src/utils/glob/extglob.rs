//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::exit;
use super::{parse, compare_one, make_prefix_strings, Wildcard};

pub fn ext_paren(cands: &mut Vec<String>, prefix: char, patterns: &Vec<String>) {
    match prefix {
        '?' => ext_question(cands, patterns),
        '*' => ext_zero_or_more(cands, patterns),
        '+' => ext_more_than_zero(cands, patterns),
        '@' => ext_once(cands, patterns),
        '!' => ext_not(cands, patterns),
        _   => exit::internal("unknown extglob prefix"),
    }
}

fn ext_question(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = cands.clone();
    for p in patterns {
        let mut tmp = cands.clone();
        parse(p, true).iter().for_each(|w| compare_one(&mut tmp, &w));
        ans.append(&mut tmp);
    }
    *cands = ans;
}

fn ext_zero_or_more(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = vec![];
    let mut tmp = cands.clone();
    let mut len = tmp.len();

    while len > 0 {
        ans.extend(tmp.clone());
        ext_once(&mut tmp, patterns);
        for a in &ans {
            tmp.retain(|t| a.as_str() != t.as_str());
        }

        len = tmp.len();
    }
    *cands = ans;
}

fn ext_more_than_zero(cands: &mut Vec<String>, patterns: &Vec<String>) {//TODO: buggy
    let mut ans: Vec<String> = vec![];
    let mut tmp: Vec<String> = cands.clone();
    let mut len = tmp.len();

    while len > 0  {
        ext_once(&mut tmp, patterns);

        for a in &ans {
            tmp.retain(|t| a.as_str() != t.as_str());
        }
        ans.extend(tmp.clone());
        len = tmp.len();
    }
    *cands = ans;
}

fn ext_once(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = vec![];
    for p in patterns {
        let mut tmp = cands.clone();
        parse(p, true).iter().for_each(|w| compare_one(&mut tmp, &w));
        ans.append(&mut tmp);
    }
    *cands = ans;
}

fn ext_not(cands: &mut Vec<String>, patterns: &Vec<String>) {
    let mut ans = vec![];
    for cand in cands.iter_mut() {
        for prefix in make_prefix_strings(cand)  {
            if ! ext_once_exact_match(&prefix, patterns) {
                ans.push(cand[prefix.len()..].to_string());
            }
        }
    }
    *cands = ans;
}

fn ext_once_exact_match(cand: &String, patterns: &Vec<String>) -> bool {
    let mut tmp = vec![cand.clone()];
    ext_once(&mut tmp, patterns);
    tmp.iter().any(|t| t == "")
}

pub fn scanner_ext_paren(remaining: &str) -> (usize, Option<Wildcard>) {
    let prefix = match remaining.chars().nth(0) {
        Some(c) => c, 
        None => return (0, None),
    };

    if "?*+@!".find(prefix) == None 
    || remaining.chars().nth(1) != Some('(') {
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

        next_nest = "?*+@!".find(c) != None;

        if c == ')' {
            match nest {
                0 => return {
                    patterns.push(chars.iter().collect());
                    (len, Some(Wildcard::ExtGlob(prefix, patterns)) )
                },
                _ => nest -= 1,
            }
        }

        chars.push(c);
    }

    (0, None)
}
