//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod terminal;

use nix::unistd;
use self::terminal::Terminal;

pub struct ShellCore {
    flags: String,
    term: Option<Terminal>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            flags: String::new(),
            term: None,
        };

        if unistd::isatty(0) == Ok(true) {
            core.flags += "i";
            core.term = Some(Terminal::new());
        }

        core
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.find(flag) != None
    }
}
