//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd;

pub struct ShellCore {
    flags: String,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            flags: String::new(),
        };

        if unistd::isatty(0)
                   .expect("sush: isatty error") {
            core.flags += "i";
        }

        core
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.find(flag) != None
    }
}
