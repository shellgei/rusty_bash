//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;
use std::process::exit;
use std::env;

pub struct Flags {
    pub v: bool,
    pub x: bool,
    pub i: bool,
    pub d: bool,
}

impl Flags {
    pub fn new() -> Flags {
        Flags{
            v: false, 
            x: false,
            i: false,
            d: false,
        }
    }
}

pub struct History {
    pub commandline: String,
    pub charwidths: Vec<u8>, 
}

pub struct ShellCore {
    pub internal_commands: HashMap<String, fn(args: &Vec<String>) -> i32>,
    pub vars: HashMap<&'static str, String>,
    pub history: Vec<History>,
    pub flags: Flags,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut conf = ShellCore{
            internal_commands: HashMap::new(),
            vars: HashMap::new(),
            history: Vec::new(),
            flags: Flags::new(),
        };

        conf.internal_commands.insert("exit".to_string(), Self::exit);
        conf.internal_commands.insert("cd".to_string(), Self::pwd);

        conf
    }

    pub fn exit(_args: &Vec<String>) -> i32 {
        exit(0);
    }

    pub fn pwd(_args: &Vec<String>) -> i32 {
            match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                _        => panic!("Cannot get current dir"),
            }
            0
    }

    pub fn get_internal_command(&self, name: &String) -> Option<fn(args: &Vec<String>) -> i32> {
        if self.internal_commands.contains_key(name) {
            Some(self.internal_commands[name])
        }else{
            None
        }
    }
}
