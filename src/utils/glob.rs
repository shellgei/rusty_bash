//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod comparator;

#[derive(Debug)]
pub enum GlobElem {
    Symbol(char),
    OneOf(bool, Vec<char>), //bool: false if ! or ^ exist
    Normal(String),
}

pub fn compare(word: &String, pattern: &[GlobElem]) -> bool {
    comparator::shave_word(word, pattern).iter().any(|c| c.is_empty())
}

pub fn parse(pattern: &str) -> Vec<GlobElem> {
    let mut remaining = pattern.to_string();
    let mut ans = vec![];

    while ! remaining.is_empty() {
        if eat_asterisk_or_question(&mut remaining, &mut ans)
        || eat_bracket(&mut remaining, &mut ans)
        || eat_escaped_char(&mut remaining, &mut ans)
        || eat_string(&mut remaining, &mut ans) {
            continue;
        }

        ans.push( GlobElem::Normal( remaining.remove(0).to_string() ) );
    }

    ans
}

fn eat_asterisk_or_question(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if pattern.starts_with("*") || pattern.starts_with("?") {
        ans.push( GlobElem::Symbol( pattern.remove(0) ) );
        return true;
    }
    false
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
            ans.push( GlobElem::OneOf(!not, inner) );
            *pattern = pattern.split_off(len);
            return true;
        }else{
            inner.push(c);
        }
    }

    false
}

fn eat_escaped_char(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if ! pattern.starts_with("\\") {
        return false;
    }

    if pattern.len() == 1 {
        ans.push( GlobElem::Normal( consume(pattern, 1) ) );
        return true;
    }
    pattern.remove(0);

    let len = pattern.chars().next().unwrap().len_utf8();
    ans.push( GlobElem::Normal( consume(pattern, len) ) );
    true
}

fn eat_string(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    let mut len = 0;
    for c in pattern.chars() {
        if "@!+*?[\\".contains(c) {
            break;
        }
        len += c.len_utf8();
    }

    if len == 0 {
        return false;
    }

    ans.push( GlobElem::Normal( consume(pattern, len) ));
    true
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining.split_off(cutpos);

    cut
}
