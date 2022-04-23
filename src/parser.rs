//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::any::Any;
use super::elements::{CommandWithArgs, Arg};

pub struct ReadingText {
    pub remaining: String,
    pub from_lineno: u32,
    pub to_lineno: u32,
    pub pos_in_line: u32,
}

// job or function comment or blank (finally) 
pub fn top_level_element(text: &mut ReadingText) -> Box<dyn Any> {
    let retText = ReadingText{
        remaining: "".to_string(),
        from_lineno: 0,
        to_lineno: 0,
        pos_in_line: 0,
    };

    //only a command is recognized currently
    if let Some(result) = command_with_args(text.remaining.clone()) {
        text.remaining = "".to_string();
        return Box::new(result)
    }
    Box::new(0)
}

pub fn command_with_args(line: String) -> Option<CommandWithArgs> {
    let mut ans = CommandWithArgs{
                     args: vec!(),
                     text: line.clone(),
                     text_pos: 0};

    let words: Vec<String> = line
        .trim()
        .split(" ")
        .map(|x| x.to_string())
        .collect();

    for w in words {
        ans.args.push(Arg{text: w.clone(), text_pos: 0});
    };

    if ans.args[0].text.len() > 0 {
        Some(ans)
    }else{
        None
    }
}

/*
pub fn arg(line: String) -> (Option<Arg>, String) {
}
*/
