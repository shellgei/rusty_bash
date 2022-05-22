//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::evaluator::{TextPos};
use crate::evaluator_args::{Arg, SubArg, SubArgBraced, ArgElem, SubArgSingleQuoted, SubArgDoubleQuoted, SubArgVariable};
use crate::parser::single_char_delimiter;

fn exist(ch: char, chars: &str) -> bool{
    if let Some(_) = chars.to_string().find(ch) {
        true
    }else{
        false
    }
}

fn check_head(text: &String, chars: &str) -> bool{
    if let Some(ch) = text.chars().nth(0) {
        exist(ch, chars)
    }else{
        false
    }
}

// single quoted arg or double quoted arg or non quoted arg 
pub fn arg(text: &mut Feeder) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
        subargs: vec!(),
    };

    while let Some(result) = subarg(text) {
        ans.text += &(*result).text();
        //ans.pos.length += (*result).get_length();
        ans.subargs.push(result);
    };

    Some(ans)
}

pub fn subarg(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = subarg_variable_braced(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_variable_non_braced(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_braced(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_normal(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_single_qt(text) {
        return Some(Box::new(a));
    }else if let Some(a) = subarg_double_qt(text) {
        return Some(Box::new(a));
    }
    None
}

pub fn arg_in_brace(text: &mut Feeder) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
        subargs: vec!(),
    };

    if check_head(&text.remaining, ",}"){ // zero length arg
        let tmp = SubArg{
            text: "".to_string(),
            pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
        };
        ans.subargs.push(Box::new(tmp));
        return Some(ans);
    };

    while let Some(result) = subarg_in_brace(text) {
        ans.text += &(*result).text();
        //ans.pos.length += (*result).get_length();
        ans.subargs.push(result);
    };

    Some(ans)
}

pub fn subarg_in_brace(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a)      = subarg_braced(text)          {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)       {Some(Box::new(a))}
    else if let Some(a) = subarg_double_qt(text)       {Some(Box::new(a))}
    else if let Some(a) = subarg_normal_in_brace(text) {Some(Box::new(a))}
    else{None}
}

pub fn subarg_normal(text: &mut Feeder) -> Option<SubArg> {
    if check_head(&text.remaining, " \n\t\"';"){
        return None;
    };

    let mut first = true;
    let mut pos = 0;
    let mut escaped = false;
    for ch in text.remaining.chars() {
        if escaped || (!escaped && ch == '\\') {
            pos += ch.len_utf8();
            escaped = !escaped;
            first = false;
            continue;
        };

        if exist(ch, " \n\t;'\"") || (!first && ch == '{') {
            let ans = SubArg{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        };

        pos += ch.len_utf8();
        first = false;
    };

    None
}

pub fn subarg_normal_in_brace(text: &mut Feeder) -> Option<SubArg> {
    if check_head(&text.remaining, ",}"){
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

        if exist(ch, ",}{") {
            let ans = SubArg{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        }
        pos += ch.len_utf8();
    };

    None
}

pub fn subarg_single_qt(text: &mut Feeder) -> Option<SubArgSingleQuoted> {
    if !check_head(&text.remaining, "'"){
        return None;
    };

    let mut pos = 1;
    for ch in text.remaining[1..].chars() {
        if ch != '\'' {
            pos += ch.len_utf8();
        }else{
            pos += 1;
            let ans = SubArgSingleQuoted{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        };
    };

    None
}

/* parser for a string such as "aaa${var}" */
pub fn subarg_double_qt(text: &mut Feeder) -> Option<SubArgDoubleQuoted> {
    let backup = text.clone();

    let mut ans = SubArgDoubleQuoted {
        text: "".to_string(),
        pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
        subargs: vec!(),
    };

    if let Some(_) = single_char_delimiter(text, '"') {
    }else{
        return None;
    }

    loop {
        if let Some(a) = subarg_variable_braced(text) {
            ans.subargs.push(Box::new(a));
        }else if let Some(a) = subarg_variable_non_braced(text) {
            ans.subargs.push(Box::new(a));
        }else if let Some(a) = string_in_double_qt(text) {
            ans.subargs.push(Box::new(a));
        }else{
            break;
        };
    }

    if let Some(_) = single_char_delimiter(text, '"') {
    }else{
        text.rewind(backup);
        return None;
    }

    let mut text = "\"".to_string();
    for a in &ans.subargs {
        text += &a.text();
    }
    ans.text = text + "\"";

    Some(ans)
}

pub fn string_in_double_qt(text: &mut Feeder) -> Option<SubArg> {
    if check_head(&text.remaining, "\""){
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

        if exist(ch, "\"$") {
        //if ch == '"' || ch == '$' {
            let ans = SubArg{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        };
        pos += ch.len_utf8();
    };

    None
}

pub fn subarg_variable_non_braced(text: &mut Feeder) -> Option<SubArgVariable> {
    if !check_head(&text.remaining, "$") ||
       text.remaining.chars().nth(1) == Some('{') {
        return None;
    };

    let mut pos = 1;
    for ch in text.remaining[1..].chars() {
        if let Some(_) = " {,;\n".find(ch) {
            let ans = SubArgVariable{
                    text: text.remaining[0..pos].to_string(),
                    pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
                 };

            text.pos_in_line += pos as u32;
            text.remaining = text.remaining[pos..].to_string();
            return Some(ans);
        };
        pos += ch.len_utf8();
    };

    None
}

pub fn subarg_variable_braced(text: &mut Feeder) -> Option<SubArgVariable> {
    if text.remaining.chars().nth(0) != Some('$') ||
       text.remaining.chars().nth(1) != Some('{') {
        return None;
    }

    let mut pos = 2;
    for ch in text.remaining[2..].chars() {
        pos += ch.len_utf8();
        if ch != '}' {
            continue;
        }

        let ans = SubArgVariable{
            text: text.remaining[0..pos].to_string(),
            pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
        };

        text.pos_in_line += pos as u32;
        text.remaining = text.remaining[pos..].to_string();
        return Some(ans);
    };

    None
}

pub fn subarg_braced(text: &mut Feeder) -> Option<SubArgBraced> {
    if let Some(_) = single_char_delimiter(text, '{') {
    }else{
        return None;
    };
    
    let mut ans = SubArgBraced {
        text: "{".to_string(),
        pos: TextPos{lineno: text.from_lineno, pos: text.pos_in_line},
        args: vec!(),
    };

    while let Some(arg) = arg_in_brace(text) {
        ans.text += &arg.text.clone();
        //ans.pos.length += arg.pos.length;
        ans.args.push(arg); 
        if let Some(_) = single_char_delimiter(text, ',') {
            ans.text += ",";
            //ans.pos.length += 1;
            continue;
        }else if let Some(_) = single_char_delimiter(text, '}') {
            ans.text += "}";
            //ans.pos.length += 1;
            break;
        };
    };

    Some(ans)
}
