//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod history;
mod parameter;

use nix::unistd;
use std::collections::HashMap;

pub struct ShellCore {
    flags: String,
    parameters: HashMap<String, String>,
    rewritten_history: HashMap<usize, String>,
    pub history: Vec<String>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            flags: String::new(),
            parameters: HashMap::new(),
            rewritten_history: HashMap::new(),
            history: vec![],
        };

        if unistd::isatty(0) == Ok(true) {
            core.flags += "i";
            core.set_param("PS1", r"\[\033[01;32m\]\u@\h\[\033[00m\]:\[\033[01;35m\]\w\[\033[00m\]🍣 ");
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
