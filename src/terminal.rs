//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::io;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

pub struct Terminal { }

impl Terminal {
    pub fn new() -> Terminal {
        Terminal { }
    }

    pub fn show_prompt(&mut self, core: &mut ShellCore, mode: &str) -> usize {
        let prompt = core.get_param_ref(mode);
        print!("{} ", prompt);
        io::stdout().flush().unwrap();
        UnicodeWidthStr::width(prompt)
    }
}

