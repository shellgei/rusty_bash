//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::element_list::ControlOperator;

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
    }else if "?*@$#!-:".chars().any(|c| c == text.nth(from)) {
        return from+1;
    };

    let mut pos = from;
    for ch in text.chars_after(from) {
        if !((ch >= '0' && ch <= '9') || (ch >= 'A' && ch <= 'Z') 
        || (ch >= 'a' && ch <= 'z') || ch == '_' || ch == '-') {
            break;
        }
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_control_op(text: &Feeder, from: usize) -> (usize, Option<ControlOperator> ) {
    if text.len() > from + 2  {
        if text.compare(from, ";;&") {
            return (from + 2, Some(ControlOperator::SemiSemiAnd));
        }
    }

    if text.len() > from + 1  {
        if text.compare(from, "||") {
            return (from + 2, Some(ControlOperator::Or));
        }else if text.compare(from, "&&") {
            return (from + 2, Some(ControlOperator::And));
        }else if text.compare(from, ";;") {
            return (from + 2, Some(ControlOperator::DoubleSemicolon));
        }else if text.compare(from, ";&") {
            return (from + 2, Some(ControlOperator::SemiAnd));
        }else if text.compare(from, "|&") {
            return (from + 2, Some(ControlOperator::PipeAnd));
        }
    }

    if text.len() > from  {
        if text.compare(from, "&") {
            return (from + 1, Some(ControlOperator::BgAnd));
        } else if text.compare(from, "\n") {
            return (from + 1, Some(ControlOperator::NewLine));
        } else if text.compare(from, "|") {
            return (from + 1, Some(ControlOperator::Pipe));
        } else if text.compare(from, ";") {
            return (from + 1, Some(ControlOperator::Semicolon));
        }
    }

    (from , None)
}

//TODO: rewrite to return a value of ControlOperator
pub fn scanner_end_of_pipeline(text: &Feeder, from: usize) -> usize {
    if text.compare(from, "&") && ! text.compare(from, "&&") { //background process
            return scanner_while(text, from+1, " ;\n");
    }

    if text.compare(from, "||") || text.compare(from, "&&") {
        return from+2;
    }else if text.compare(from, ";;") {
        return from;
    }

    let n = scanner_while(text, from, ";\n");
    if n > from {
        return n;
    }

    if text.nth_is(from, "#") {
        return scanner_until(text, from, "\n");
    }

    return from;
}

pub fn scanner_end_of_com(text: &Feeder, from: usize) -> usize {
    if text.nth_is(from, "|") {
        if text.len() > from+1 { //check of ||
            if text.nth_is(from+1, "|") {
                return from;
            }
        }
        return from+1;
    }

    if text.nth_is(from, "#") {
        return scanner_until(text, from, "\n");
    }

    return from;
}

pub fn scanner_end_paren(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, ")") {
        return from+1;
    }
    return from;
}

/* TODO: these scanners should be summarized. */ 
pub fn scanner_start_paren(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, "(") {
        return from+1;
    }
    return from;
}

pub fn scanner_start_brace(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, "{") {
        return from+1;
    }
    return from;
}

pub fn scanner_integer(text: &Feeder, from: usize) -> usize {
    if text.len() == from {
        return from;
    }

    let mut pos = from;
    if text.nth(from) == '-' {
        pos += 1;
    }

    for ch in text.chars_after(pos) {
        if ch < '0' || ch > '9' {
            break;
        }

        pos += 1;
    }

    if text.nth(from) == '-' && pos == from+1 {
        from
    }else{
        pos
    }
}
