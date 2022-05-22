//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::BashElem;
use super::bash_elements::{CommandWithArgs, ArgDelimiter, Eoc};
use super::arg_elements::{DelimiterInArg};
use crate::parser_args::arg;
use crate::ShellCore;
use crate::Feeder;
use crate::debuginfo::DebugInfo;


// job or function comment or blank (finally) 
pub fn top_level_element(text: &mut Feeder, _config: &mut ShellCore) -> Option<Box<dyn BashElem>> {
    if text.len() == 0 {
        return None;
    };

    let backup = text.clone();

    if let Some(eoc) = end_of_command(text) {
        return Some(Box::new(eoc));
    };

    //only a command is recognized currently
    if let Some(result) = command_with_args(text) {
        return Some(Box::new(result));
    }

    text.rewind(backup);
    None
}

pub fn command_with_args(text: &mut Feeder) -> Option<CommandWithArgs> {
    let mut ans = CommandWithArgs{
        elems: vec!(),
        text: "".to_string(),
    };

    if let Some(result) = delimiter(text){
        ans.text += &result.text;
        ans.elems.push(Box::new(result));
    }

    while let Some(result) = arg(text) {
        ans.text += &result.text;
        ans.elems.push(Box::new(result));

        if let Some(result) = delimiter(text){
            ans.text += &result.text;
            ans.elems.push(Box::new(result));
        }else if let Some(result) = end_of_command(text){
            ans.text += &result.text;
            ans.elems.push(Box::new(result));
            break;
        }
    }

    if ans.elems.len() > 0 {
        Some(ans)
    }else{
        None
    }
}

pub fn delimiter(text: &mut Feeder) -> Option<ArgDelimiter> {
    let mut length = 0;
    for ch in text.chars() {
        if ch == ' ' || ch == '\t' {
            length += 1;
        }else{
            break;
        };
    };

    if length != 0 {
        let ans = ArgDelimiter{
            text: text.consume(length),
            debug: DebugInfo::init(text),
        };
        return Some(ans);
    };

    None
}

pub fn arg_delimiter(text: &mut Feeder, symbol: char) -> Option<ArgDelimiter> {
    if text.nth(0) == symbol {
        Some( ArgDelimiter{ text: text.consume(1), debug: DebugInfo::init(&text),})
    }else{
        None
    }
}

pub fn delimiter_in_arg(text: &mut Feeder, symbol: char) -> Option<DelimiterInArg> {
    if text.nth(0) == symbol {
        Some( DelimiterInArg{ text: text.consume(1), debug: DebugInfo::init(&text),})
    }else{
        None
    }
}

pub fn end_of_command(text: &mut Feeder) -> Option<Eoc> {
    if text.len() == 0 {
        return None;
    };

    let ch = text.nth(0);
    if ch == ';' || ch == '\n' {
        let ans = Eoc{
            text: text.consume(1),
            debug: DebugInfo::init(&text),
        };

        return Some(ans);
    };

    None
}
