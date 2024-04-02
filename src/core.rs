//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod parameter;

use nix::unistd;
use std::collections::HashMap;

pub struct ShellCore {
    flags: String,
    pub history: Vec<Vec<char>>,
    parameters: HashMap<String, String>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            flags: String::new(),
            history: vec![vec![]],
            parameters: HashMap::new(),
        };

        if unistd::isatty(0) == Ok(true) {
            core.flags += "i";
            core.set_param("PS1", "🍣 ");
            core.set_param("PS2", "> ");
        }

        core
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.find(flag) != None
    }
}
