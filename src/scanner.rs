//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;

pub fn scanner_subarg_no_quote(text: &Feeder, start: usize) -> usize {
    scanner_escaped_string(text, " \n\t\"';{}", start)
}

pub fn scanner_subvalue_no_quote(text: &Feeder, start: usize) -> usize {
    scanner_escaped_string(text, " \n\t\"';", start)
}

pub fn scanner_varname(text: &Feeder, start: usize) -> usize {
    let mut pos = start;
    for ch in text.chars_after(start) {
        if !((ch >= '0' && ch <= '9') ||(ch >= 'A' && ch <= 'Z') 
        || (ch >= 'a' && ch <= 'z') || ch == '_'){
            break;
        }
        pos += 1;
    }
    pos
}

fn scanner_escaped_string(text: &Feeder, ng_chars: &str, start: usize) -> usize {
    let mut pos = start;
    let mut escaped = false;
    for ch in text.chars_after(start) {
        if escaped || ch == '\\' {
            escaped = !escaped;
        }else if let Some(_) = ng_chars.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}
