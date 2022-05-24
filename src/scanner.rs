//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;

fn scanner_escaped_string(text: &Feeder, from: usize, to: &str) -> usize {
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

pub fn scanner_string(text: &Feeder, from: usize, to: &str) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = to.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_subarg_no_quote(text: &Feeder, from: usize) -> usize {
    scanner_escaped_string(text, from, " \n\t\"';{}")
}

pub fn scanner_subvalue_no_quote(text: &Feeder, from: usize) -> usize {
    scanner_escaped_string(text, from, " \n\t\"';")
}

pub fn scanner_varname(text: &Feeder, from: usize) -> usize {
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

pub fn scanner_subarg_normal_in_brace(text: &Feeder, from: usize) -> usize {
    scanner_escaped_string(text, from, ",{}")
}
