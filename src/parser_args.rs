//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::debuginfo::{DebugInfo};
use crate::elems_in_arg::{SubArgNonQuoted, SubArgBraced, ArgElem, SubArgSingleQuoted, SubArgDoubleQuoted, SubArgVariable, SubArgCommandExp};
use crate::scanner::*;

pub fn subarg(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)          {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandExp::parse(text)    {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)      {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text)        {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse(text)     {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)           {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text)  {Some(Box::new(a))}
    else                                                   {None}
}

pub fn subvalue(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)          {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandExp::parse(text)    {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)      {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse2(text)    {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)           {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text)  {Some(Box::new(a))}
    else                                                   {None}
}

pub fn subarg_in_brace(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)         {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)     {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text)       {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)          {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text) {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse3(text)   {Some(Box::new(a))}
    else{None}
}

pub fn subarg_single_qt(text: &mut Feeder) -> Option<SubArgSingleQuoted> {
    if !text.match_at(0, "'"){
        return None;
    };

    let pos = scanner_until(text, 1, "'");
    Some(SubArgSingleQuoted{text: text.consume(pos+1), pos: DebugInfo::init(text)})
}

pub fn string_in_double_qt(text: &mut Feeder) -> Option<SubArgNonQuoted> {
    if text.nth(0) == '"' {
        return None;
    };

    let pos = scanner_until_escape(text, 0, "\"$");
    Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text)})
}

/*
pub fn SubArgVariable::parse(text: &mut Feeder) -> Option<SubArgVariable> {
    if !(text.nth(0) == '$') || text.nth(1) == '{' {
        return None;
    };

    let pos = scanner_varname(&text, 1);
    Some(
        SubArgVariable{
            text: text.consume(pos),
            pos: DebugInfo::init(text),
        })
}

pub fn SubArgVariable::parse2(text: &mut Feeder) -> Option<SubArgVariable> {
    if !(text.nth(0) == '$' && text.nth(1) == '{') {
        return None;
    }

    let pos = scanner_varname(&text, 2);
    if text.nth(pos) == '}' {
        Some( SubArgVariable{ text: text.consume(pos+1), pos: DebugInfo::init(text) })
    }else{
        None
    }
}
*/
