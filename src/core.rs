//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
use std::collections::HashMap;

pub struct ShellCore {
    pub history: Vec<String>,
    pub builtins: HashMap<String, fn(&mut ShellCore, args: &mut Vec<String>) -> i32>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let conf = ShellCore{
            history: Vec::new(),
            builtins: HashMap::new(),
        };

        conf
    }
}
