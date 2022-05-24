//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::utils::exist;

pub fn scanner_subarg_no_quote(text: &Feeder, start: usize) -> usize {
    let mut pos = start;
    let mut escaped = false;
    for ch in text.chars_after(start) {
        if escaped || ch == '\\' {
            escaped = !escaped;
        }else if exist(ch, " \n\t\"';{}") {
            break;
        };
            
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_subvalue_no_quote(text: &Feeder, start: usize) -> usize {
    let mut pos = start;
    let mut escaped = false;
    for ch in text.chars_after(start) {
        if escaped || ch == '\\' {
            escaped = !escaped;
        }else if exist(ch, " \n\t\"';") {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
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

