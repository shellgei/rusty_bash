//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::BashElem;
use super::evaluator::{TextPos, CommandWithArgs, Delim, Eoc, Empty};
use crate::parser_args::arg;
use crate::ShellCore;

pub struct ReadingText {
    pub remaining: String,
    pub from_lineno: u32,
    pub to_lineno: u32,
    pub pos_in_line: u32,
}

// job or function comment or blank (finally) 
pub fn top_level_element(text: &mut ReadingText, _config: &mut ShellCore) -> Box<dyn BashElem> {
    //only a command is recognized currently
    if let Some(result) = command_with_args(text) {
        text.remaining = "".to_string();
        return Box::new(result)
    }

    let e = Empty{};
    Box::new(e)
}

pub fn command_with_args(text: &mut ReadingText) -> Option<CommandWithArgs> {
    let mut ans = CommandWithArgs{
                     elems: vec!(),
                     text: text.remaining.clone(),
                     text_pos: 0};

    if let Some(result) = delimiter(text){
        ans.elems.push(Box::new(result));
    }

    while let Some(result) = arg(text) {
        ans.elems.push(Box::new(result));

        if let Some(result) = delimiter(text){
            ans.elems.push(Box::new(result));
        }else if let Some(result) = end_of_command(text){
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

pub fn delimiter(text: &mut ReadingText) -> Option<Delim> {
    let mut length = 0;
    for ch in text.remaining.chars() {
        if ch == ' ' || ch == '\t' {
            length += 1;
        }else{
            break;
        }
    };

    if length != 0 {
        let ans = Delim{
            text: text.remaining[0..length].to_string(),
            pos: TextPos{
                lineno: text.from_lineno,
                pos: text.pos_in_line,
                length: length
            }
        };

        text.pos_in_line += length as u32;
        text.remaining = text.remaining[length..].to_string();
        return Some(ans);
    };

    None
}

pub fn end_of_command(text: &mut ReadingText) -> Option<Eoc> {
    if text.remaining.len() == 0 {
        return None;
    };

    let ch = &text.remaining[0..1];
    if ch == ";" || ch == "\n" {
        let ans = Eoc{
            text: ch.to_string(),
            pos: TextPos{
                lineno: text.from_lineno,
                pos: text.pos_in_line,
                length: 1
            }
        };

        text.pos_in_line += 1;
        text.remaining = text.remaining[1..].to_string();
        return Some(ans);
    };

    None
}
