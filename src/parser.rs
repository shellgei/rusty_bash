//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::CString;

// job or function comment or blank (finally) 
pub fn top_level_element(line: String) -> (bool, Box<[CString]>) {
    //only a command is recognized currently
    let words: Vec<CString> = line
        .trim()
        .split(" ")
        .map(|x| CString::new(x).unwrap())
        .collect::<Vec<_>>();

    let array = words.into_boxed_slice();
    (true, array)
}
