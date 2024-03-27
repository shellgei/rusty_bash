//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd;

pub struct ShellCore {
    flags: String,
}

fn is_interactive() -> bool {
    match unistd::isatty(0) {
        Ok(result) => result,
        Err(err) => panic!("{}", err),
    }
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            flags: String::new(),
        };

        if is_interactive() {
            core.flags += "i";
        }

        core
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.find(flag) != None
    }
}
