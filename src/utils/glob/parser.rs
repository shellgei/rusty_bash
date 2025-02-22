//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::{MetaChar, GlobElem, extglob};

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

fn cut_charclass(pattern: &mut String) -> Option<MetaChar> {
    for c in vec!["alnum", "alpha", "ascii", "blank", "cntrl",
                  "digit", "graph", "lower", "print", "punct",
                  "space", "upper", "word", "xdigit"] {
        if pattern.starts_with(&("[:".to_owned() + c + ":]")) {
            return Some(MetaChar::CharClass(consume(pattern, c.len() + 4)));
        }
    }

    None
}

fn cut_metachar(pattern: &mut String) -> Option<MetaChar> {
    if pattern.starts_with("]") {
        return None;
    }

    if pattern.starts_with("[:") {
        if let Some(cls) = cut_charclass(pattern) {
            return Some(cls);
        }
    }

    if pattern.starts_with("\\") {
        if pattern.len() > 1 {
            let ch = pattern.chars().nth(1).unwrap();
            *pattern = pattern.split_off(ch.len_utf8() + 1);
            return Some(MetaChar::Normal(ch));
        }else{
            *pattern = pattern.split_off(1);
            return None;
        }
    }

    if pattern.len() > 2
    && pattern.chars().nth(1) == Some('-')
    && pattern.chars().nth(2) != Some(']') {
        let f = pattern.chars().nth(0).unwrap();
        let t = pattern.chars().nth(2).unwrap();
        *pattern = pattern.split_off(f.len_utf8() + 1 + t.len_utf8());
        return Some(MetaChar::Range(f, t));
    }

    if pattern.len() > 0 {
        let ch = pattern.chars().nth(0).unwrap();
        *pattern = pattern.split_off(ch.len_utf8());
        return Some(MetaChar::Normal(ch));
    }

    None
}

fn eat_bracket(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if ! pattern.starts_with("[") {
        return false;
    }
    
    let bkup = pattern.clone();
    let not = pattern.starts_with("[^") || pattern.starts_with("[!");
    let len = if not {2} else {1};
    let mut inner = vec![];

    *pattern = pattern.split_off(len);
    while pattern.len() > 0 {
        if pattern.starts_with("]") {
            *pattern = pattern.split_off(1);
            ans.push( GlobElem::OneOf(!not, inner) );
            return true;
        }

        if let Some(p) = cut_metachar(pattern) {
            inner.push(p);
        }
    }

    *pattern = bkup;
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

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining.split_off(cutpos);

    cut
}
