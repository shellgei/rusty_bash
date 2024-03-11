//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::word::Word;
use glob;

pub fn eval(word: &mut Word, core: &mut ShellCore) {
    match glob::glob(&word.text) {
        Ok(paths) => {
            for path in paths {
                dbg!("{:?}", path.expect("").display());
            }
        },
        _ => return,
    };
}
