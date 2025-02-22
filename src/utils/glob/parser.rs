//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::{CharClass, GlobElem, extglob};

fn eat_one_char(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if pattern.starts_with("*") || pattern.starts_with("?") {
        ans.push( GlobElem::Symbol(pattern.remove(0))  );
        return true;
    }
    false
}

fn eat_escaped_char(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if ! pattern.starts_with("\\") {
        return false;
    }

    if pattern.len() == 1 {
        ans.push( GlobElem::Normal(pattern.remove(0).to_string()) );
        return true;
    }
    pattern.remove(0);

    let len = pattern.chars().nth(0).unwrap().len_utf8();
    ans.push( GlobElem::Normal( consume(pattern, len) ) );
    true
}

fn eat_bracket(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if ! pattern.starts_with("[") {
        return false;
    }
    
    let not = pattern.starts_with("[^") || pattern.starts_with("[!");
    let mut len = if not {2} else {1};
    let mut escaped = false;
    let mut inner = vec![];

    for c in pattern[len..].chars() {
        len += c.len_utf8();

        if escaped {
            inner.push(c); 
            escaped = false;
        }else if c == '\\' {
            escaped = true;
        }else if c == ']' {
            let expand_inner = expand_range_representation(&inner);
            ans.push( GlobElem::OneOf(!not, expand_inner) );
            *pattern = pattern.split_off(len);
            return true;
        }else{
            inner.push(c);
        }
    }

    false
}

fn eat_extglob(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    let (len, extparen) = extglob::scan(pattern);
    if len > 0 {
        *pattern = pattern.split_off(len);
        ans.push(extparen.unwrap());
        return true;
    }
    false
}

fn eat_chars(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    let mut len = 0;
    for c in pattern.chars() {
        if "@!+*?[\\".find(c) != None {
            break;
        }
        len += c.len_utf8();
    }

    if len == 0 {
        return false;
    }

    let s = consume(pattern, len);
    ans.push( GlobElem::Normal(s) );
    true
}

pub fn parse(pattern: &str, extglob: bool) -> Vec<GlobElem> {
    let pattern = pattern.to_string();
    let mut remaining = pattern.to_string();
    let mut ans = vec![];

    while remaining.len() > 0 {
        if (extglob && eat_extglob(&mut remaining, &mut ans) )
        || eat_bracket(&mut remaining, &mut ans) 
        || eat_one_char(&mut remaining, &mut ans) 
        || eat_escaped_char(&mut remaining, &mut ans) 
        || eat_chars(&mut remaining, &mut ans) {
            continue;
        }

        let s = consume(&mut remaining, 1);
        ans.push( GlobElem::Normal(s) );
    }

    ans
}

fn expand_range_representation(chars: &Vec<char>) -> Vec<CharClass> {
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
            ans.push(CharClass::Normal(*c));
            from = Some(*c);
        }
    }

    if hyphen {
        ans.push(CharClass::Normal('-'));
    }

    ans
}

fn expand_range(from: &Option<char>, to: &char) -> Vec<CharClass> {
    if from.is_none() {
        return vec![CharClass::Normal(*to)];
    }

    let from = from.unwrap();

    let mut ans = vec![];

    if ('0' <= from && from <= *to && *to <= '9')
    || ('a' <= from && from <= *to && *to <= 'z')
    || ('A' <= from && from <= *to && *to <= 'Z') {
        let mut ch = from;
        while ch <= *to {
            ans.push(CharClass::Normal(ch));
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
