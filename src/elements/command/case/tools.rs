//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug)]
enum Wildcard {
    Normal(String),
    Asterisk,
    Question,
    //OneOf(Vec<char>),
    //NotOneOf(Vec<char>),
}

pub fn compare(word: &String, pattern: &str) -> bool {
    let wildcards = parse(pattern);
    let mut candidates = vec![word.to_string()];

    for w in wildcards {
        match w {
            Wildcard::Normal(s) => compare_normal(&mut candidates, &s),
            Wildcard::Asterisk  => asterisk(&mut candidates),
            Wildcard::Question  => question(&mut candidates),
        }
    }

    candidates.iter().any(|c| c == "")
}

pub fn compare_normal(cands: &mut Vec<String>, s: &String) {
    let mut ans = vec![];

    for c in cands.into_iter() {
        if ! c.starts_with(s) {
            continue;
        }
        
        ans.push(c[s.len()..].to_string());
    }

    *cands = ans;
}

pub fn asterisk(cands: &mut Vec<String>) {
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

pub fn question(cands: &mut Vec<String>) {
    let mut ans = vec![];
    for cand in cands.into_iter() {
        match cand.chars().nth(0) {
            Some(c) => {
                let len = c.len_utf8();
                ans.push(cand[len..].to_string());
            },
            _ => {},
        }
    }
    *cands = ans;
}

fn parse(pattern: &str) -> Vec<Wildcard > {
    let pattern = pattern.to_string();
    let mut remaining = pattern.to_string();

    let mut ans = vec![];

    while remaining.len() > 0 {
        match scanner_escaped_char(&remaining) {
            0 => {}, 
            len => {
                let mut s = consume(&mut remaining, len);
                s.remove(0);
                ans.push( Wildcard::Normal(s) );
                continue;
            },
        }

        if remaining.starts_with("*") {
            consume(&mut remaining, 1);
            ans.push( Wildcard::Asterisk );
            continue;
        }else if remaining.starts_with("?") {
            consume(&mut remaining, 1);
            ans.push( Wildcard::Question );
            continue;
        }

        let len = scanner_chars(&remaining);
        let s = consume(&mut remaining, len);
        ans.push( Wildcard::Normal(s) );
    }

    ans
}

fn scanner_escaped_char(remaining: &str) -> usize {
    if ! remaining.starts_with("\\") {
        return 0;
    }

    match remaining.chars().nth(1) {
        None    => 1,
        Some(c) => 1 + c.len_utf8(),
    }
}

fn scanner_chars(remaining: &str) -> usize {
    let mut ans = 0;
    for c in remaining.chars() {
        if c == '*' || c == '?' {
            return ans;
        }

        ans += c.len_utf8();
    }
    ans
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining[cutpos..].to_string();

    cut
}
