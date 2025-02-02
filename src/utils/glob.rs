//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod comparator;

#[derive(Debug)]
pub enum GlobElem {
    Symbol(char),
    OneOf(bool, Vec<char>), //bool: false if ! or ^ exist
    Normal(String),
}

pub fn compare(word: &String, pattern: &Vec<GlobElem>) -> bool {
    comparator::shave_word(word, pattern).iter().any(|c| c == "")
}

pub fn parse(pattern: &str) -> Vec<GlobElem> {
    let mut remaining = pattern.to_string();
    let mut ans = vec![];

    while remaining.len() > 0 {
        if eat_asterisk_or_question(&mut remaining, &mut ans)
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

fn eat_escaped_char(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
    if ! pattern.starts_with("\\") {
        return false;
    }

    if pattern.len() == 1 {
        ans.push( GlobElem::Normal( consume(pattern, 1) ) );
        return true;
    }
    pattern.remove(0);

    let len = pattern.chars().nth(0).unwrap().len_utf8();
    ans.push( GlobElem::Normal( consume(pattern, len) ) );
    true
}

fn eat_string(pattern: &mut String, ans: &mut Vec<GlobElem>) -> bool {
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

    ans.push( GlobElem::Normal( consume(pattern, len) ));
    true
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let back = remaining.split_off(cutpos);
    let front = remaining.clone();
    *remaining = back;
    front
}
