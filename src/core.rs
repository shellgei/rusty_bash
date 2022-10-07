//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;
use std::fs::File;
use std::env;
use crate::core_shopts::Shopts;
use crate::job::Job;

pub struct ShellCore {
    pub builtins: HashMap<String, fn(&mut ShellCore, args: &mut Vec<String>) -> i32>,
    pub functions: HashMap<String, String>,
    pub arrays: HashMap<String, Vec<String>>,
    pub vars: HashMap<String, String>,
    pub args: Vec<String>,
    pub aliases: HashMap<String, String>,
    pub history: Vec<String>,
    pub flags: String,
    pub jobs: Vec<Job>,
    pub in_double_quot: bool,
    pub pipeline_end: String,
    pub script_file: Option<File>,
    pub return_enable: bool,
    pub return_flag: bool,
    pub shopts: Shopts, 
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut conf = ShellCore{
            builtins: HashMap::new(),
            functions: HashMap::new(),
            arrays: HashMap::new(),
            vars: HashMap::new(),
            args: vec![],
            aliases: HashMap::new(),
            history: Vec::new(),
            flags: String::new(),
            jobs: vec!(Job::new(&"".to_string(), &vec![])),
            in_double_quot: false,
            pipeline_end: String::new(),
            script_file: None,
            return_flag: false,
            return_enable: false,
            shopts: Shopts::new(),
        };

        conf.set_var("?", &0.to_string());

        // Builtins: they are implemented in core_builtins.rs. 
        conf.builtins.insert(".".to_string(), Self::source);
        conf.builtins.insert(":".to_string(), Self::true_);
        conf.builtins.insert("alias".to_string(), Self::alias);
        conf.builtins.insert("builtin".to_string(), Self::builtin);
        conf.builtins.insert("cd".to_string(), Self::cd);
        conf.builtins.insert("eval".to_string(), Self::eval);
        conf.builtins.insert("exit".to_string(), Self::exit);
        conf.builtins.insert("export".to_string(), Self::export);
        conf.builtins.insert("false".to_string(), Self::false_);
        conf.builtins.insert("history".to_string(), Self::history);
        conf.builtins.insert("jobs".to_string(), Self::jobs);
        conf.builtins.insert("pwd".to_string(), Self::pwd);
        conf.builtins.insert("set".to_string(), Self::set);
        conf.builtins.insert("shift".to_string(), Self::shift);
        conf.builtins.insert("true".to_string(), Self::true_);
        conf.builtins.insert("read".to_string(), Self::read);
        conf.builtins.insert("return".to_string(), Self::return_);
        conf.builtins.insert("shopt".to_string(), Self::shopt);
        conf.builtins.insert("source".to_string(), Self::source);
        conf.builtins.insert("wait".to_string(), Self::wait);

        conf.builtins.insert("glob_test".to_string(), Self::glob_test);

        conf
    }

    pub fn set_var(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
    }

    pub fn get_var(&self, key: &str) -> String {
        if let Ok(n) = key.parse::<usize>() {
            if self.args.len() > n {
                return self.args[n].clone();
            }
        }

        if key == "-" {
            return self.flags.clone();
        }

        if key == "#" {
            return (self.args.len() - 1).to_string();
        }

        if key == "@" {
            if self.args.len() == 1 {
                return "".to_string();
            }

            return self.args[1..].to_vec().join(" ");
        }

        if key == "*" {
            if self.args.len() == 1 {
                return "".to_string();
            }

            if self.in_double_quot {
                //if let Some(ch) = self.get_var(&"IFS".to_string()).chars().nth(0){
                if let Some(ch) = self.get_var("IFS").chars().nth(0){
                    return self.args[1..].to_vec().join(&ch.to_string());
                }
            }

            return self.args[1..].to_vec().join(" ");
        }

        if let Some(s) = self.vars.get(&key as &str){
            return s.to_string();
        };

        if let Ok(s) = env::var(&key) {
            return s.to_string();
        };

        "".to_string()
    }

    pub fn get_function(&mut self, name: &String) -> Option<String> {
        if self.functions.contains_key(name) {
            if let Some(s) = self.functions.get(name) {
                return Some(s.clone());
            }
        }

        None
    }

    pub fn get_internal_command(&self, name: &String) 
        -> Option<fn(&mut ShellCore, args: &mut Vec<String>) -> i32> {
        if self.builtins.contains_key(name) {
            Some(self.builtins[name])
        }else{
            None
        }
    }

    pub fn has_flag(&self, flag: char) -> bool {
        if let Some(_) = self.flags.find(flag) {
            return true;
        }
        false
    }
}
