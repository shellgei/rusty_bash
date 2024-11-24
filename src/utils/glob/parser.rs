//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::GlobElem;
use super::extglob;

pub fn parse(pattern: &str, extglob: bool) -> Vec<GlobElem> {
    let pattern = pattern.to_string();
    let mut remaining = pattern.to_string();

    let mut ans = vec![];

    while remaining.len() > 0 {
        let len =  scan_escaped_char(&remaining);
        if len > 0 {
            let mut s = consume(&mut remaining, len);
            s.remove(0);
            ans.push( GlobElem::Normal(s) );
            continue;
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
            ans.push( GlobElem::Asterisk );
            continue;
        }else if remaining.starts_with("?") {
            consume(&mut remaining, 1);
            ans.push( GlobElem::Question );
            continue;
        }

        let len = scan_chars(&remaining);
        if len > 0 {
            let s = consume(&mut remaining, len);
            ans.push( GlobElem::Normal(s) );
            continue;
        }

        let s = consume(&mut remaining, 1);
        ans.push( GlobElem::Normal(s) );
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

fn scan_bracket(remaining: &str) -> (usize, GlobElem) {
    if ! remaining.starts_with("[") {
        return (0, GlobElem::OneOf(vec![]) );
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
        }else if c == '\\' {
            escaped = true;
        }else if c == ']' {
            let expand_chars = expand_range_representation(&chars);
            return match not {
                false => (len, GlobElem::OneOf(expand_chars) ),
                true  => (len, GlobElem::NotOneOf(expand_chars) ),
            };
        }else{
            chars.push(c);
        }
    }

    (0, GlobElem::OneOf(vec![]) )
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

    if hyphen {
        ans.push('-');
    }

    ans
}

fn expand_range(from: &Option<char>, to: &char) -> Vec<char> {
    if from.is_none() {
        return vec![*to];
    }

    let from = from.unwrap();

    let mut ans = vec![];

    if ('0' <= from && from <= *to && *to <= '9')
    || ('a' <= from && from <= *to && *to <= 'z')
    || ('A' <= from && from <= *to && *to <= 'Z') {
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
    *remaining = remaining.split_off(cutpos);

    cut
}
