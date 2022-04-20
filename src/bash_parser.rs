//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::CString;
use super::bash_elements::{CommandWithArgs,Tree};

// job or function comment or blank (finally) 
pub fn top_level_element(line: String) -> Option<CommandWithArgs> {
    //only a command is recognized currently
    command_with_args(line)
}

pub fn command_with_args(line: String) -> Option<CommandWithArgs> {
    let mut ans = CommandWithArgs{tree: Tree::new(), args: Box::new([])};
    ans.tree.text = line.clone();

    let words: Vec<String> = line
        .trim()
        .split(" ")
        .map(|x| x.to_string())
        .collect();


    for w in words {
        let tree = Tree{elems: Vec::new(), text: w.clone(), text_pos: 0};
        ans.tree.elems.push(tree);
    };

    let raw_words: Vec<CString> = line
        .trim()
        .split(" ")
        .map(|x| CString::new(x).unwrap())
        .collect();

    ans.args = raw_words.into_boxed_slice();

    Some(ans)
}
