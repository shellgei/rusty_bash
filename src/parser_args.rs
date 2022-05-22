//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::debuginfo::{DebugInfo};
use crate::single_command_elems::{Arg};
use crate::arg_elems::{SubArg, SubArgBraced, ArgElem, SubArgSingleQuoted, SubArgDoubleQuoted, SubArgVariable};
use crate::parser::{arg_delimiter,delimiter_in_arg};

fn exist(ch: char, chars: &str) -> bool{
    if let Some(_) = chars.to_string().find(ch) {
        true
    }else{
        false
    }
}


// single quoted arg or double quoted arg or non quoted arg 
pub fn arg(text: &mut Feeder) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subargs: vec!(),
    };

    while let Some(result) = subarg(text) {
        ans.text += &(*result).text();
        ans.subargs.push(result);
    };

    Some(ans)
}

pub fn subarg(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = subarg_variable_braced(text)          {Some(Box::new(a))}
    else if let Some(a) = subarg_variable_non_braced(text) {Some(Box::new(a))}
    else if let Some(a) = subarg_braced(text)              {Some(Box::new(a))}
    else if let Some(a) = subarg_normal(text)              {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)           {Some(Box::new(a))}
    else if let Some(a) = subarg_double_qt(text)           {Some(Box::new(a))}
    else                                                   {None}
}

pub fn arg_in_brace(text: &mut Feeder) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subargs: vec!(),
    };

    if text.check_head(",}"){ // zero length arg
        let tmp = SubArg{
            text: "".to_string(),
            pos: DebugInfo::init(text),
        };
        ans.subargs.push(Box::new(tmp));
        return Some(ans);
    };

    while let Some(result) = subarg_in_brace(text) {
        ans.text += &(*result).text();
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
    if text.check_head(" \n\t\"';"){
        return None;
    };

    let mut first = true;
    let mut pos = 0;
    let mut escaped = false;
    for ch in text.chars() {
        if escaped || (!escaped && ch == '\\') {
            pos += ch.len_utf8();
            escaped = !escaped;
            first = false;
            continue;
        };

        if exist(ch, " \n\t;'\"") || (!first && ch == '{') {
            let ans = SubArg{
                    text: text.consume(pos),
                    pos: DebugInfo::init(text),
                 };
            return Some(ans);
        };

        pos += ch.len_utf8();
        first = false;
    };

    None
}

pub fn subarg_normal_in_brace(text: &mut Feeder) -> Option<SubArg> {
    if text.check_head(",}"){
        return None;
    };

    let mut pos = 0;
    let mut escaped = false;
    for ch in text.chars() {
        if escaped || (!escaped && ch == '\\') {
            pos += ch.len_utf8();
            escaped = !escaped;
            continue;
        };

        if exist(ch, ",}{") {
            let ans = SubArg{
                    text: text.consume(pos),
                    pos: DebugInfo::init(text),
                 };

            return Some(ans);
        }
        pos += ch.len_utf8();
    };

    None
}

pub fn subarg_single_qt(text: &mut Feeder) -> Option<SubArgSingleQuoted> {
    if !text.check_head("'"){
        return None;
    };

    let mut pos = 1;
    for ch in text.chars_after(1) {
        if ch != '\'' {
            pos += ch.len_utf8();
        }else{
            pos += 1;
            let ans = SubArgSingleQuoted{
                    text: text.consume(pos),
                    pos: DebugInfo::init(text),
                 };

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
        pos: DebugInfo::init(text),
        subargs: vec!(),
    };

    if let Some(_) = delimiter_in_arg(text, '"') {
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

    if let Some(_) = delimiter_in_arg(text, '"') {
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
    if text.check_head("\""){
        return None;
    };

    let mut pos = 0;
    let mut escaped = false;
    for ch in text.chars() {
        if escaped || (!escaped && ch == '\\') {
            pos += ch.len_utf8();
            escaped = !escaped;
            continue;
        };

        if exist(ch, "\"$") {
        //if ch == '"' || ch == '$' {
            let ans = SubArg{
                    text: text.consume(pos),
                    pos: DebugInfo::init(text),
                 };

            return Some(ans);
        };
        pos += ch.len_utf8();
    };

    None
}

pub fn subarg_variable_non_braced(text: &mut Feeder) -> Option<SubArgVariable> {
    if !text.check_head("$") || text.nth(1) == '{' {
        return None;
    };

    let mut pos = 1;
    for ch in text.chars_after(1) {
        if let Some(_) = " {,;\n".find(ch) {
            return Some(
                SubArgVariable{
                    text: text.consume(pos),
                    pos: DebugInfo::init(text),
                 })
        };
        pos += ch.len_utf8();
    };

    None
}

pub fn subarg_variable_braced(text: &mut Feeder) -> Option<SubArgVariable> {
    if text.chars().nth(0) != Some('$') ||
       text.chars().nth(1) != Some('{') {
        return None;
    }

    let mut pos = 2;
    for ch in text.chars_after(2) {
        pos += ch.len_utf8();
        if ch != '}' {
            continue;
        }

        return Some( SubArgVariable{
            text: text.consume(pos),
            pos: DebugInfo::init(text),
        } )
    };

    None
}

pub fn subarg_braced(text: &mut Feeder) -> Option<SubArgBraced> {
    if let Some(_) = delimiter_in_arg(text, '{') {
    }else{
        return None;
    };
    
    let mut ans = SubArgBraced {
        text: "{".to_string(),
        pos: DebugInfo::init(text),
        args: vec!(),
    };

    while let Some(arg) = arg_in_brace(text) {
        ans.text += &arg.text.clone();
        ans.args.push(arg); 
        if let Some(_) = arg_delimiter(text, ',') {
            ans.text += ",";
            continue;
        }else if let Some(_) = delimiter_in_arg(text, '}') {
            ans.text += "}";
            break;
        };
    };

    Some(ans)
}
