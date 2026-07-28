//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::{GlobElem, MetaChar, extglob};

fn eat_one_char(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if pattern.starts_with("*") || pattern.starts_with("?") {
        ans.push(GlobElem::Symbol(pattern.remove(0)));
        return true;
    }
    false
}

fn eat_escaped_char(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if !pattern.starts_with("\\") {
        return false;
    }

    if pattern.len() == 1 {
        ans.push(GlobElem::Normal(pattern.remove(0).to_string()));
        return true;
    }
    //ans.push( GlobElem::Normal(pattern.remove(0).to_string()) );
    pattern.remove(0);

    let len = pattern.chars().next().unwrap().len_utf8();
    ans.push(GlobElem::Normal(consume(pattern, len)));
    true
}

fn cut_charclass(pattern: &mut String) -> Option<MetaChar> {
    for c in vec![
        "alnum", "alpha", "ascii", "blank", "cntrl", "digit", "graph", "lower", "print", "punct",
        "space", "upper", "word", "xdigit",
    ] {
        if pattern.starts_with(&("[:".to_owned() + c + ":]")) {
            return Some(MetaChar::CharClass(consume(pattern, c.len() + 4)));
        }
    }

    None
}

fn cut_col_symbol(pattern: &mut String) -> Option<MetaChar> {
    if ! pattern.starts_with("[.") {
        return None;
    }

    let mut len = 2;
    let mut last_dot = false;
    for c in pattern.chars().skip(2) {
        if c == ']' && last_dot {
            let whole_part = consume(pattern, len + 1);
            let whole_len = whole_part.len();
            return Some(MetaChar::CollatingSymbol(whole_part[2..whole_len-2].to_string()));
        }

        if c == '.' {
            last_dot = true;
        }else if ! c.is_ascii_graphic() {
            return None;
        }

        len += c.len_utf8();
    }

    None
}

fn cut_equiv_class(pattern: &mut String) -> Option<MetaChar> {
    if ! pattern.starts_with("[=") {
        return None;
    }

    let len;
    let ch;
    if let Some(c) = pattern.chars().nth(2) {
        ch = c;
        len = ch.len_utf8();
    }else{
        return None;
    }

    if let Some('=') = pattern.chars().nth(3) {
    }else{
        return None;
    }

    if let Some(']') = pattern.chars().nth(4) {
    }else{
        return None;
    }

    consume(pattern, len + 4);
    Some(MetaChar::EquivalenceClass(ch))
}

fn cut_metachar(pattern: &mut String) -> Option<MetaChar> {
    if pattern.starts_with("]") {
        return None;
    }

    if pattern.starts_with("[:")
        && let Some(cls) = cut_charclass(pattern)
    {
        return Some(cls);
    }

    if pattern.starts_with("[.")
        && let Some(cls) = cut_col_symbol(pattern)
    {
        return Some(cls);
    }

    if pattern.starts_with("[=")
        && let Some(cls) = cut_equiv_class(pattern)
    {
        return Some(cls);
    }

    if pattern.starts_with("\\") {
        if pattern.len() > 1 {
            let ch = pattern.chars().nth(1).unwrap();
            *pattern = pattern.split_off(ch.len_utf8() + 1);
            return Some(MetaChar::Normal(ch));
        } else {
            *pattern = pattern.split_off(1);
            return None;
        }
    }

    if pattern.len() > 2
        && pattern.chars().nth(1) == Some('-')
        && pattern.chars().nth(2) != Some(']')
    {
        let f = pattern.chars().next().unwrap();
        let t = pattern.chars().nth(2).unwrap();
        *pattern = pattern.split_off(f.len_utf8() + 1 + t.len_utf8());
        return Some(MetaChar::Range(f, t));
    }

    if !pattern.is_empty() {
        let ch = pattern.chars().next().unwrap();
        *pattern = pattern.split_off(ch.len_utf8());
        return Some(MetaChar::Normal(ch));
    }

    None
}

fn false_charclass_check(inner: &Vec<MetaChar>) -> bool {
    if inner.len() < 3 {
        return false;
    }

    if let MetaChar::Normal(':') = inner[1] {
        if let Some(MetaChar::Normal(':')) = inner.last() {
            let len = inner.len() - 1;
            for e in &inner[2..len] {
                if let MetaChar::Normal(c) = e {
                    if ! c.is_ascii_alphanumeric() {
                        return false;
                    }
                }
            }

            return true;
        }
    }

    false
}

fn oneof_to_string(inner: &Vec<MetaChar>) -> String {
    let mut ans = String::new();

    for e in inner.iter() {
        match e {
            MetaChar::Normal(c) => ans.push(*c),
            _ => {},
        }
    }

    ans.push(']');
    ans
}

fn col_to_range(inner: &mut Vec<MetaChar>) -> bool {
    let len = inner.len();
    if len < 3 {
        return false;
    }

    for s in 0..len-2 {
        let range = if let MetaChar::Normal('-') = inner[s+1] {
            match (&inner[s], &inner[s+2]) {
                ( MetaChar::CollatingSymbol(sc), MetaChar::Normal(ec) ) => {
                    let sc = col_symbol_to_char(sc);
                    Some(MetaChar::Range(sc, *ec))
                },
                ( MetaChar::Normal(sc), MetaChar::CollatingSymbol(ec) ) => {
                    let ec = col_symbol_to_char(ec);
                    Some(MetaChar::Range(*sc, ec))
                },
                ( MetaChar::CollatingSymbol(sc), MetaChar::CollatingSymbol(ec) ) => {
                    let sc = col_symbol_to_char(sc);
                    let ec = col_symbol_to_char(ec);
                    Some(MetaChar::Range(sc, ec))
                },
                _ => {None},
            }
        }else{
            None
        };

        if range.is_some() {
            for _ in 0..3 {
                inner.remove(s);
            }
            inner.insert(s, range.unwrap());
            return true;
        }
    }

    false
}

fn eat_bracket(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if !pattern.starts_with("[") {
        return false;
    }

    let bkup = pattern.clone();
    let not = pattern.starts_with("[^") || pattern.starts_with("[!");
    let len = if not { 2 } else { 1 };
    let mut inner = vec![];

    *pattern = pattern.split_off(len);
    while !pattern.is_empty() {
        if pattern.starts_with("]") {
            *pattern = pattern.split_off(1);

            if false_charclass_check(&inner) {
                let s = oneof_to_string(&inner);
                ans.push(GlobElem::Normal(s));
                return true;
            }

            while col_to_range(&mut inner) {}

            ans.push(GlobElem::OneOf(!not, inner));
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
        if "@!+*?[\\".find(c).is_some() {
            break;
        }
        len += c.len_utf8();
    }

    if len == 0 {
        return false;
    }

    let s = consume(pattern, len);
    ans.push(GlobElem::Normal(s));
    true
}

pub fn parse(pattern: &str, extglob: bool, nomatchcase: bool) -> Vec<GlobElem> {
    let mut pattern = pattern.to_string();
    if nomatchcase {
        pattern = pattern.to_lowercase();
    }
    let mut remaining = pattern.to_string();
    let mut ans = vec![];

    while !remaining.is_empty() {
        if (extglob && eat_extglob(&mut remaining, &mut ans))
            || eat_bracket(&mut remaining, &mut ans)
            || eat_one_char(&mut remaining, &mut ans)
            || eat_escaped_char(&mut remaining, &mut ans)
            || eat_chars(&mut remaining, &mut ans)
        {
            continue;
        }

        let s = consume(&mut remaining, 1);
        ans.push(GlobElem::Normal(s));
    }

    ans
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining.split_off(cutpos);

    cut
}

fn col_symbol_to_char(symbol: &str) -> char { //TODO: complete!
    if symbol == "hyphen" {
        return '-';
    }else if symbol == "space" {
        return ' ';
    }else if symbol == "tab" {
        return '\t';
    }else if symbol == "newline" {
        return '\n';
    }

    symbol.chars().nth(0).unwrap()
}
