//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ReadingText;
use crate::evaluator::{TextPos};
use crate::evaluator_args::{Arg, SubArg, ArgElem};
use crate::evaluator_args::{SubArgSingleQuoted, SubArgDoubleQuoted};

// single quoted arg or double quoted arg or non quoted arg 
pub fn arg(text: &mut ReadingText) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line, length: 0},
        subargs: vec!(),
    };

    while let Some(result) = subarg(text) {
        ans.text += &(*result).get_text();
        ans.pos.length += (*result).get_length();
        ans.subargs.push(result);
    };

    Some(ans)
}

pub fn subarg(text: &mut ReadingText) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = subarg_normal(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_braced(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_single_qt(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_double_qt(text) {
        return Some(Box::new(a));
    }
    None
}

pub fn subarg_normal(text: &mut ReadingText) -> Option<SubArg> {
    if let Some(ch) = text.remaining.chars().nth(0) {
        if ch == ' ' || ch == '\n' || ch == '\t' || ch == '"' || ch == '\'' || ch == ';' || ch == '{' {
            return None;
        };
    }else{
        return None;
    };

    let mut pos = 0;
    let mut escaped = false;
    for ch in text.remaining.chars() {
        if escaped || (!escaped && ch == '\\') {
            pos += ch.len_utf8();
            escaped = !escaped;
            continue;
        };

        if ch == ' ' || ch == '\n' || ch == '\t' || ch == ';' || ch == '\'' || ch == '"' || ch == '{' {
            let ans = SubArg{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line, length: pos},
                    braced: false,
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        }else{
            pos += ch.len_utf8();
        };
    };

    None
}

pub fn subarg_single_qt(text: &mut ReadingText) -> Option<SubArgSingleQuoted> {
    if text.remaining.chars().nth(0) != Some('\'') {
        return None;
    }

    let mut pos = 1;
    for ch in text.remaining[1..].chars() {
        if ch != '\'' {
            pos += ch.len_utf8();
        }else{
            pos += 1;
            let ans = SubArgSingleQuoted{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line, length: pos},
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        };
    };

    None
}

pub fn subarg_double_qt(text: &mut ReadingText) -> Option<SubArgDoubleQuoted> {
    if text.remaining.chars().nth(0) != Some('"') {
        return None;
    }

    let mut pos = 1;
    let mut escaped = false;
    for ch in text.remaining[1..].chars() {
        if escaped || (!escaped && ch == '\\') {
            pos += ch.len_utf8();
            escaped = !escaped;
            continue;
        };

        if ch != '"' {
            pos += ch.len_utf8();
        }else{
            pos += 1;
            let ans = SubArgDoubleQuoted{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line, length: pos},
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        };
    };

    None
}

pub fn subarg_braced(text: &mut ReadingText) -> Option<SubArg> {
    if let Some(ch) = text.remaining.chars().nth(0) {
        if ch != '{' {
            return None;
        };
    }else{
        return None;
    };

    let mut pos = 0;
    let mut escaped = false;
    let mut comma = false;
    for ch in text.remaining.chars() {
        if escaped || (!escaped && ch == '\\') {
            pos += ch.len_utf8();
            escaped = !escaped;
            continue;
        };

        if ch == ',' && !escaped {
            comma = true;
        }

        if ch == ' ' || ch == '\n' || ch == '\t' || ch == ';' || ch == '\'' || ch == '"' {
            let ans = SubArg{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line, length: pos},
                    braced: false,
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        }else if ch == '}'{
            pos += ch.len_utf8();
            let ans = SubArg{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line, length: pos},
                    braced: comma,
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        }else{
            pos += ch.len_utf8();
        };
    };

    None
}

pub fn expand_brace(text: &String) -> Vec<String>{
    let mut ans = vec!();
    let mut tmp = "".to_string();
    let stripped = text[1..text.len()-1].to_string().clone();
    let mut escaped = false;
    for ch in stripped.chars() {
        if escaped {
            escaped = false;
            tmp.push(ch);
        }else if ch == '\\' {
            escaped = true;
        }else if ch == ',' {
            ans.push(tmp);
            tmp = "".to_string();
        }else{
            tmp.push(ch);
        };
    }
    ans.push(tmp);
    ans
}
