//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::any::Any;
use super::elements::{CommandWithArgs,Arg};

// job or function comment or blank (finally) 
//pub fn top_level_element(line: String) -> Option<CommandWithArgs> {
pub fn top_level_element(line: String) -> Box<dyn Any> {
    //only a command is recognized currently
    match command_with_args(line) {
        Some(result) => Box::new(result),
        None => panic!("!!"),
    }
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

    Some(ans)
}
