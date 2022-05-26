//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::elems_executable::{Substitutions, Executable, BlankPart, CommandWithArgs};
use super::elems_in_command::{ArgDelimiter, Eoc};
use super::elems_in_arg::{DelimiterInArg};
use crate::parser_args::{arg,substitution};
use crate::ShellCore;
use crate::Feeder;
use crate::debuginfo::DebugInfo;
use crate::scanner::{scanner_end, scanner_delimiter};

// job or function comment or blank (finally) 
pub fn top_level_element(text: &mut Feeder, _config: &mut ShellCore) -> Option<Box<dyn Executable>> {
    if text.len() == 0 {
        return None;
    };

    let backup = text.clone();

    if let Some(result) = blank_part(text)       {return Some(Box::new(result));}
    if let Some(result) = substitutions(text)    {return Some(Box::new(result));}
    if let Some(result) = command_with_args(text){return Some(Box::new(result));}

    text.rewind(backup);
    None
}

pub fn blank_part(text: &mut Feeder) -> Option<BlankPart> {
    let mut ans = BlankPart::new();

    loop {
        if let Some(d) = delimiter(text)          {ans.push(Box::new(d));}
        else if let Some(e) = end_of_command(text){ans.push(Box::new(e));}
        else{break;};
    };

    BlankPart::return_if_valid(ans)
}

pub fn substitutions(text: &mut Feeder) -> Option<Substitutions> {
    let backup = text.clone();
    let mut ans = Substitutions::new();

    while let Some(result) = substitution(text) {
        ans.push(Box::new(result));

        if let Some(result) = delimiter(text){
            ans.push(Box::new(result));
        }
    }

    if let Some(result) = end_of_command(text){
        ans.push(Box::new(result));
    }else{
        text.rewind(backup);
        return None;
    }

    Substitutions::return_if_valid(ans)
}


pub fn command_with_args(text: &mut Feeder) -> Option<CommandWithArgs> {
    let backup = text.clone();
    let mut ans = CommandWithArgs::new();

    while let Some(s) = substitution(text) {
        ans.push_vars(s);

        if let Some(d) = delimiter(text){
            ans.push_elems(Box::new(d));
        }
    }

    while let Some(a) = arg(text, true) {
        ans.push_elems(Box::new(a));

        if let Some(d) = delimiter(text){
            ans.push_elems(Box::new(d));
        }

        if let Some(e) = end_of_command(text){
            ans.push_elems(Box::new(e));
            break;
        }
    }

    CommandWithArgs::return_if_valid(ans, text, backup)
}

pub fn delimiter(text: &mut Feeder) -> Option<ArgDelimiter> {
    let pos = scanner_delimiter(text, 0);
    ArgDelimiter::return_if_valid(text, pos)
}

pub fn arg_delimiter(text: &mut Feeder, symbol: char) -> Option<ArgDelimiter> {
    if text.nth(0) == symbol {
        Some( ArgDelimiter{ text: text.consume(1), debug: DebugInfo::init(&text)})
    }else{
        None
    }
}

pub fn delimiter_in_arg(text: &mut Feeder, symbol: char) -> Option<DelimiterInArg> {
    if text.nth(0) == symbol {
        Some( DelimiterInArg{ text: text.consume(1), debug: DebugInfo::init(&text)})
    }else{
        None
    }
}

pub fn end_of_command(text: &mut Feeder) -> Option<Eoc> {
    if text.len() == 0 {
        return None;
    };

    let pos = scanner_end(text, 0);
    if pos == 0 {
        return None;
    };

    Some( Eoc{
        text: text.consume(pos),
        debug: DebugInfo::init(&text),
    })
}
