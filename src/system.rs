//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;

pub struct Flags {
    pub v: bool,
    pub x: bool,
    pub i: bool,
}

impl Flags {
    pub fn new() -> Flags {
        Flags{
            v: false, 
            x: false,
            i: false,
        }
    }
}

pub struct Config {
    pub vars: HashMap<&'static str, String>,
    pub flags: Flags,
}

impl Config {
    pub fn new() -> Config {
        Config{
            vars: HashMap::new(),
            flags: Flags::new(),
        }
    }
}
