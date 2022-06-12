//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;

pub fn scanner_until_escape(text: &Feeder, from: usize, to: &str) -> usize {
    let mut pos = from;
    let mut escaped = false;
    for ch in text.chars_after(from) {
        if escaped || ch == '\\' {
            escaped = !escaped;
        }else if let Some(_) = to.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_while(text: &Feeder, from: usize, chars: &str) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = chars.find(ch) {
            pos += ch.len_utf8();
        }else{
            break;
        };
    }
    pos
}

pub fn scanner_until(text: &Feeder, from: usize, to: &str) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = to.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_varname(text: &Feeder, from: usize) -> usize {
    if text.len() == from {
        return from;
    }else if text.nth(from) == '?' {
        return from+1;
    };

    let mut pos = from;
    for ch in text.chars_after(from) {
        if !((ch >= '0' && ch <= '9') ||(ch >= 'A' && ch <= 'Z') 
        || (ch >= 'a' && ch <= 'z') || ch == '_'){
            break;
        }
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_end_of_com(text: &Feeder, from: usize) -> usize {
    if text.match_at(from, ";\n|") {
        return from+1;
    }

    if text.match_at(from, "#") {
        return scanner_until(text, from, "\n");
    }

    return from;
}

pub fn scanner_end_paren(text: &Feeder, from: usize) -> usize {
    if text.match_at(from, ")") {
        return from+1;
    }
    return from;
}
