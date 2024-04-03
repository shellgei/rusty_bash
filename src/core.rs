//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod parameter;

use nix::unistd;
use std::collections::HashMap;

pub struct ShellCore {
    flags: String,
    parameters: HashMap<String, String>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            flags: String::new(),
            parameters: HashMap::new(),
        };

        if unistd::isatty(0) == Ok(true) {
            core.flags += "i";
            core.set_param("PS1", "🍣 ");
            core.set_param("PS2", "> ");
        }

        let home = core.get_param_ref("HOME").to_string();
        core.set_param("HISTFILE", &(home + "/.bash_history"));
        core.set_param("HISTFILESIZE", "2000");

        core
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.find(flag) != None
    }
}
