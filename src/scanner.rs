//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::element_list::{ControlOperator, Reserved};

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

pub fn scanner_blank(text: &Feeder, from: usize) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = " \t".find(ch) {
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

pub fn scanner_name_or_parameter(text: &Feeder, from: usize) -> usize {
    let ans = scanner_parameter(text, from);

    if ans == 0 {
        scanner_name(text, from)
    }else{
        ans
    }
}

pub fn scanner_parameter(text: &Feeder, from: usize) -> usize {
    if text.len() < from {
        return from;
    }

    if "?*@$#!-:".chars().any(|c| c == text.nth(from)) { //special parameters
        return from+1;
    };

    let mut pos = from;
    for ch in text.chars_after(from) { //position parameter
        if ch < '0' || '9' < ch {
            break;
        }
        pos += 1;
    }
    pos
}

pub fn scanner_name(text: &Feeder, from: usize) -> usize {
    if text.len() <= from {
        return from;
    }

    let h = &text.nth(0);
    if !((*h >= 'A' && *h <= 'Z') || (*h >= 'a' && *h <= 'z') || *h == '_') {
        return from;
    }

    if text.len() == from+1 {
        return from+1;
    }

    let mut ans = from+1;
    for c in text.chars_after(from+1) {
        if !((c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') 
        || (c >= 'a' && c <= 'z') || c == '_') {
            break;
        }
        ans += 1;
    }

    return ans;
}

pub fn scanner_reserved(text: &Feeder) -> (usize, Option<Reserved> ) {
    if text.starts_with("function"){
        return (8, Some(Reserved::Function));
    }

    (0, None)
}

pub fn scanner_control_op(text: &Feeder) -> (usize, Option<ControlOperator> ) {
    let mut op = None;
    let mut pos = 0;

    if text.len() > 2  {
        pos = 3;
        op = if text.compare(0, ";;&") {
            Some(ControlOperator::SemiSemiAnd)
        }else{
            None
        };
    }

    if op == None && text.len() > 1  {
        pos = 2;
        op = if text.compare(0, "||") {
            Some(ControlOperator::Or)
        }else if text.compare(0, "&&") {
            Some(ControlOperator::And)
        }else if text.compare(0, ";;") {
            Some(ControlOperator::DoubleSemicolon)
        }else if text.compare(0, ";&") {
            Some(ControlOperator::SemiAnd)
        }else if text.compare(0, "|&") {
            Some(ControlOperator::PipeAnd)
        }else{
            None
        };

    }

    if op == None && text.len() > 0  {
        pos = 1;
        if text.compare(0, "&") {
            if text.len() > 1 && text.compare(1, ">") {
                return (0, None)
            }
            return (1, Some(ControlOperator::BgAnd));
        } else if text.compare(0, "\n") {
            return (1, Some(ControlOperator::NewLine));
        } else if text.compare(0, "|") {
            return (1, Some(ControlOperator::Pipe));
        } else if text.compare(0, ";") {
            return (1, Some(ControlOperator::Semicolon));
        } else if text.compare(0, "(") {
            return (1, Some(ControlOperator::LeftParen));
        } else if text.compare(0, ")") {
            return (1, Some(ControlOperator::RightParen));
        }
    }

    if op != None && text.len() > pos && text.compare(pos, "\n") {
        pos += 1;
    }

    if op != None{
        return (pos, op);
    }


    (0 , None)
}

pub fn scanner_comment(text: &Feeder, from: usize) -> usize {
    if text.len() > from && text.nth_is(from, "#") {
        return scanner_until(text, from, "\n");
    }

    from
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
