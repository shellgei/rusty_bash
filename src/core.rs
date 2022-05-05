//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;
use std::process::exit;

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

pub struct ShellCore {
    pub internal_commands: HashMap<String, fn() -> i32>,
    pub vars: HashMap<&'static str, String>,
    pub flags: Flags,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut conf = ShellCore{
            internal_commands: HashMap::new(),
            vars: HashMap::new(),
            flags: Flags::new(),
        };

        conf.internal_commands.insert("exit".to_string(), Self::exit);

        conf
    }

    pub fn exit() -> i32 {
        exit(0);
    }

    pub fn exec_internal_command(f: fn() -> i32) -> i32{
        f()
    }
}
