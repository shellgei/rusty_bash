//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug)]
enum Wildcard {
    Normal(String),
    Asterisk,
    Question,
    OneOf(Vec<char>),
    NotOneOf(Vec<char>),
    ExtQuestion(String),
}

pub fn compare(word: &String, pattern: &str) -> bool {
    let wildcards = parse(pattern);
    let mut candidates = vec![word.to_string()];

    for w in wildcards {
        compare_internal(&mut candidates, &w);
    }

    candidates.iter().any(|c| c == "")
}

fn compare_internal(candidates: &mut Vec<String>, w: &Wildcard) {
    match w {
        Wildcard::Normal(s) => compare_normal(candidates, &s),
        Wildcard::Asterisk  => asterisk(candidates),
        Wildcard::Question  => question(candidates),
        Wildcard::OneOf(cs) => one_of(candidates, &cs, false),
        Wildcard::NotOneOf(cs) => one_of(candidates, &cs, true),
        Wildcard::ExtQuestion(p) => ext_question(candidates, &p),
    }
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

fn ext_question(cands: &mut Vec<String>, pattern: &String) {
    let mut backup = cands.clone();
    //let wildcards = parse(pattern);
    for w in parse(pattern) {
        compare_internal(cands, &w);
    }
    cands.append(&mut backup);
}

pub fn one_of(cands: &mut Vec<String>, cs: &Vec<char>, inverse: bool) {
    let mut ans = vec![];
    for cand in cands.into_iter() {
        if cs.iter().any(|c| cand.starts_with(*c)) ^ inverse {
            let h = cand.chars().nth(0).unwrap();
            ans.push(cand[h.len_utf8()..].to_string());
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

        let (len, wc) = scanner_ext_question(&remaining);
        if len > 0 {
            consume(&mut remaining, len);
            ans.push(wc);
            continue;
        }

        let (len, wc) = scanner_bracket(&remaining);
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

        let len = scanner_chars(&remaining);
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
        if "*?[\\".find(c) != None {
            return ans;
        }

        ans += c.len_utf8();
    }
    ans
}

fn scanner_bracket(remaining: &str) -> (usize, Wildcard) {
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
            match not {
                false => return (len, Wildcard::OneOf(chars) ),
                true  => return (len, Wildcard::NotOneOf(chars) ),
            }
        }

        chars.push(c);
    }

    (0, Wildcard::OneOf(vec![]) )
}

fn scanner_ext_question(remaining: &str) -> (usize, Wildcard) {
    if ! remaining.starts_with("?(") {
        return (0, Wildcard::ExtQuestion(String::new()) );
    }
    
    let mut chars = vec![];
    let mut len = 2;
    let mut escaped = false;
    let mut nest = 0;
    let mut next_nest = false;

    for c in remaining[len..].chars() {
        //eprintln!("{:?}, {:?}", &next_nest, &c);
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

        if next_nest && c == '(' {
            nest += 1;
        }

        next_nest = c == '?';

        if c == ')' {
            match nest {
                0 => return (len, Wildcard::ExtQuestion(chars.iter().collect()) ),
                _ => nest -= 1,
            }
        }

        chars.push(c);
    }

    (0, Wildcard::ExtQuestion(String::new()) )
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining[cutpos..].to_string();

    cut
}
