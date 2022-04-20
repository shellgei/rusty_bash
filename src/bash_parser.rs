//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::CString;
use super::bash_elements::{CommandWithArgs,Core};

// job or function comment or blank (finally) 
pub fn top_level_element(line: String) -> Option<CommandWithArgs> {
    //only a command is recognized currently
    let words: Vec<CString> = line
        .trim()
        .split(" ")
        .map(|x| CString::new(x).unwrap())
        .collect::<Vec<_>>();

    let mut ans = CommandWithArgs{core: Core::new(), args: Box::new([])};
    ans.args = words.into_boxed_slice();
    //let array = words.into_boxed_slice();
    //(true, array)
    Some(ans)
}
