//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

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
    pub flags: Flags,
}

impl Config {
    pub fn new() -> Config {
        Config{
            flags: Flags::new(),
        }
    }
}
