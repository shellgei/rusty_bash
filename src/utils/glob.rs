//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod extglob;

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
    get_eaten_candidates(word, pattern, extglob).iter().any(|c| c == "")
}

pub fn longest_match_length(word: &String, pattern: &str, extglob: bool) -> usize {
    let candidates = get_eaten_candidates(word, pattern, extglob);
    match candidates.len() {
        0 => 0,
        _ => word.len() - candidates.iter().map(|c| c.len()).min().unwrap(),
    }
}

pub fn shortest_match_length(word: &String, pattern: &str, extglob: bool) -> usize {
    let candidates = get_eaten_candidates(word, pattern, extglob);
    match candidates.len() {
        0 => 0,
        _ => word.len() - candidates.iter().map(|c| c.len()).max().unwrap(),
    }
}

fn get_eaten_candidates(word: &String, pattern: &str, extglob: bool) -> Vec<String> {
    let mut candidates = vec![word.to_string()];

    for w in parse(pattern, extglob) {
        eat(&mut candidates, &w);
    }
    candidates
}

fn eat(candidates: &mut Vec<String>, w: &Wildcard) {
    match w {
        Wildcard::Normal(s) => nonspecial(candidates, &s),
        Wildcard::Asterisk  => asterisk(candidates),
        Wildcard::Question  => question(candidates),
        Wildcard::OneOf(cs) => one_of(candidates, &cs, false),
        Wildcard::NotOneOf(cs) => one_of(candidates, &cs, true),
        Wildcard::ExtGlob(prefix, ps) => extglob::ext_paren(candidates, *prefix, &ps),
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

fn parse(pattern: &str, extglob: bool) -> Vec<Wildcard > {
    let pattern = pattern.to_string();
    let mut remaining = pattern.to_string();

    let mut ans = vec![];

    while remaining.len() > 0 {
        match scan_escaped_char(&remaining) {
            0 => {}, 
            len => {
                let mut s = consume(&mut remaining, len);
                s.remove(0);
                ans.push( Wildcard::Normal(s) );
                continue;
            },
        }

        if extglob {
            let (len, extparen) = extglob::scan(&remaining);
            if len > 0 {
                consume(&mut remaining, len);
                ans.push(extparen.unwrap());
                continue;
            }
        }

        let (len, wc) = scan_bracket(&remaining);
        if len > 0 {
            consume(&mut remaining, len);
            ans.push(wc);
            continue;
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

        let len = scan_chars(&remaining);
        if len > 0 {
            let s = consume(&mut remaining, len);
            ans.push( Wildcard::Normal(s) );
            continue;
        }

        let s = consume(&mut remaining, 1);
        ans.push( Wildcard::Normal(s) );
    }

    ans
}

fn scan_escaped_char(remaining: &str) -> usize {
    if ! remaining.starts_with("\\") {
        return 0;
    }

    match remaining.chars().nth(1) {
        None    => 1,
        Some(c) => 1 + c.len_utf8(),
    }
}

fn scan_chars(remaining: &str) -> usize {
    let mut ans = 0;
    for c in remaining.chars() {
        if "@!+*?[\\".find(c) != None {
            return ans;
        }
        ans += c.len_utf8();
    }
    ans
}

fn scan_bracket(remaining: &str) -> (usize, Wildcard) {
    if ! remaining.starts_with("[") {
        return (0, Wildcard::OneOf(vec![]) );
    }
    
    let mut chars = vec![];
    let mut len = 1;
    let mut escaped = false;
    let mut not = false;

    if remaining.starts_with("[^") || remaining.starts_with("[!") {
        not = true;
        len = 2;
    }

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

        if c == ']' {
            let expand_chars = expand_range_representation(&chars);
            match not {
                false => return (len, Wildcard::OneOf(expand_chars) ),
                true  => return (len, Wildcard::NotOneOf(expand_chars) ),
            }
        }

        chars.push(c);
    }

    (0, Wildcard::OneOf(vec![]) )
}

fn expand_range_representation(chars: &Vec<char>) -> Vec<char> {
    let mut ans = vec![];
    let mut from = None;
    let mut hyphen = false;

    for c in chars {
        if *c == '-' {
            hyphen = true;
            continue;
        }

        if hyphen {
            if ans.len() > 0 {
                ans.pop();
            }

            let mut expand = expand_range(&from, c);
            ans.append(&mut expand);
            hyphen = false;
            continue;
        }else {
            ans.push(*c);
            from = Some(*c);
        }
    }
    ans
}

fn expand_range(from: &Option<char>, to: &char) -> Vec<char> {
    if from.is_none() {
        return vec![*to];
    }

    let from = from.unwrap();

    let mut ans = vec![];

    if '0' <= from && from <= '9' 
    || 'a' <= from && from <= 'z'
    || 'A' <= from && from <= 'Z' {
        let mut ch = from;
        while ch <= *to {
            ans.push(ch);
            ch = (ch as u8 + 1) as char;
        }

    }
    ans
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining[cutpos..].to_string();

    cut
}
