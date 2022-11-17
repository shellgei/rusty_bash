//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
use std::collections::HashMap;

pub struct ShellCore {
    pub history: Vec<String>,
    pub builtins: HashMap<String, fn(&mut ShellCore, args: &mut Vec<String>) -> i32>,
    pub vars: HashMap<String, String>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut conf = ShellCore{
            history: Vec::new(),
            builtins: HashMap::new(),
            vars: HashMap::new(),
        };

        conf.builtins.insert("cd".to_string(), Self::cd);
        conf.builtins.insert("exit".to_string(), Self::exit);

        conf
    }

    pub fn get_builtin(&self, name: &String) 
        -> Option<fn(&mut ShellCore, args: &mut Vec<String>) -> i32> {
        if self.builtins.contains_key(name) {
            Some(self.builtins[name])
        }else{
            None
        }
    } 

    pub fn get_var(&self, key: &str) -> String {
        if let Some(s) = self.vars.get(&key as &str){
            return s.to_string();
        };

        "".to_string()
    }
}
