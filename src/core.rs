//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;
use std::process::exit;
use std::{io,fs,env};
use std::path::Path;
use std::fs::{OpenOptions, File};
use std::io::Write;
use crate::bash_glob::glob_match;
use crate::core_shopts::Shopts;

use crate::Script;
use crate::Feeder;

pub struct ShellCore {
    pub internal_commands: HashMap<String, fn(&mut ShellCore, args: &mut Vec<String>) -> i32>,
    pub functions: HashMap<String, String>,
    pub vars: HashMap<String, String>,
    pub args: Vec<String>,
    pub aliases: HashMap<String, String>,
    pub history: Vec<String>,
    pub flags: String,
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
            internal_commands: HashMap::new(),
            functions: HashMap::new(),
            vars: HashMap::new(),
            args: vec!(),
            aliases: HashMap::new(),
            history: Vec::new(),
            flags: String::new(),
            in_double_quot: false,
            pipeline_end: String::new(),
            script_file: None,
            return_flag: false,
            return_enable: false,
            shopts: Shopts::new(),
        };

        conf.vars.insert("?".to_string(), 0.to_string());

        conf.internal_commands.insert("exit".to_string(), Self::exit);
        conf.internal_commands.insert("pwd".to_string(), Self::pwd);
        conf.internal_commands.insert("cd".to_string(), Self::cd);
        conf.internal_commands.insert("alias".to_string(), Self::alias);
        conf.internal_commands.insert("set".to_string(), Self::set);
        conf.internal_commands.insert("read".to_string(), Self::read);
        conf.internal_commands.insert("source".to_string(), Self::source);
        conf.internal_commands.insert("return".to_string(), Self::return_);
        conf.internal_commands.insert("shopt".to_string(), Self::shopt);
        conf.internal_commands.insert("export".to_string(), Self::export);
        conf.internal_commands.insert("eval".to_string(), Self::eval);
        conf.internal_commands.insert("glob_test".to_string(), Self::glob_test);

        conf
    }

    pub fn get_var(&self, key: &String) -> String {
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
                if let Some(ch) = self.get_var(&"IFS".to_string()).chars().nth(0){
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
        if self.internal_commands.contains_key(name) {
            Some(self.internal_commands[name])
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

    /////////////////////////////////
    /* INTERNAL COMMANDS HEREAFTER */
    /////////////////////////////////
    pub fn exit(&mut self, args: &mut Vec<String>) -> i32 {
        let home = env::var("HOME").expect("HOME is not defined");
        if let Ok(mut hist_file) = OpenOptions::new().write(true)
                                   .append(true).open(home + "/.bash_history") {
            for h in &self.history {
                write!(hist_file, "{}\n", h).expect("Cannot write history");
            };
            hist_file.flush().expect("Cannot flush the history file");
        }


        if args.len() >= 2 {
            if let Ok(status) = args[1].parse::<i32>(){
                exit(status);
            }else{
                eprintln!("exit: {}: numeric argument required", args[1]);
                exit(2);
            }
        }

        if let Ok(status) = self.get_var(&"?".to_string()).to_string().parse::<i32>(){
            exit(status);
        }else{
            eprintln!("Shell internal error");
            exit(1);
        }
    }

    pub fn pwd(&mut self, _args: &mut Vec<String>) -> i32 {
        if let Some(p) = env::current_dir().expect("Cannot get current dir").to_str() {
            println!("{}", p.to_string());
            return 0;
        };

        panic!("Cannot get current dir");
    }

    pub fn cd(&mut self, args: &mut Vec<String>) -> i32 {
        if args.len() == 0 {
            eprintln!("Bug of this shell");
        }else if args.len() == 1 { //only "cd"
            let var = env::var("HOME").expect("HOME is not defined");
            args.push(var);
        }else if args.len() == 2 && args[1] == "-" { // cd -
            if let Some(old) = self.vars.get("OLDPWD") {
                args[1] = old.to_string();
            }
        };

        if let Ok(old) = env::current_dir() {
            self.vars.insert("OLDPWD".to_string(), old.display().to_string());
        };

        let path = Path::new(&args[1]);
        if env::set_current_dir(&path).is_ok() {
            if let Ok(full) = fs::canonicalize(path) {
                self.vars.insert("PWD".to_string(), full.display().to_string());
            }
            0
        }else{
            eprintln!("Not exist directory");
            1
        }
    }

    pub fn alias(&mut self, args: &mut Vec<String>) -> i32 {
        if args.len() <= 1 {
            for (k, v) in self.aliases.iter() {
                println!("alias {}='{}'", k, v);
            }
            return 0;
        }

        if let Some(com) = self.aliases.get(&args[1]) {
            println!("alias {}='{}'", &args[1], com);
            return 0;
        }

        let elems = args[1].split('=').collect::<Vec<&str>>();
        if elems.len() < 2 {
            eprintln!("bash: alias: {} not found", &args[1]);
            return 1;
        }

        self.aliases.insert(elems[0].to_string(), elems[1..].join("="));
        0
    }

    pub fn set(&mut self, args: &mut Vec<String>) -> i32 {
        if args.len() == 1 {
           for k in self.vars.keys() {
               println!("{}={}", k, self.vars[k]);
           }
           return 0;
        }

        self.args.clear();

        for a in args {
            self.args.push(a.to_string());
        }

        0
    }

    pub fn read(&mut self, args: &mut Vec<String>) -> i32 {
        let mut line = String::new();
        if io::stdin().read_line(&mut line).expect("Failed to read line") == 0 {
            return 1;
        }

        let argnum = args.len() - 1;
        if argnum < 1 {
            return 0;
        }

        let mut token = line.trim_end().split(" ").map(|s| s.to_string()).collect::<Vec<String>>();

        let last = if argnum < token.len() {
            token[argnum-1..].join(" ")
        }else if argnum == token.len() {
            token[argnum-1].clone()
        }else{
            "".to_string()
        };

        if token.len() >= argnum-1 {
            token.insert(argnum-1, last.clone());
        }else{
            while token.len() < argnum {
                token.push(last.clone());
            }
        }

        //let values = token[..argnum].to_vec();

        for (i, a) in args.iter().enumerate() {
            if i == 0 {
                continue;
            }

            self.vars.insert(a.to_string(), token[i-1].to_string());
        }

        0
    }

    pub fn source(&mut self, args: &mut Vec<String>) -> i32 {
        if args.len() < 2 {
            eprintln!("usage: source filename");
            return 1;
        }

        if args.len() > 1 {
            match fs::read_to_string(&args[1]) {
                Ok(source) => {
                    let mut feeder = Feeder::new_with(source);
                    if let Some(mut script) = Script::parse(&mut feeder, self, vec!("")) {
                        self.return_enable = true;
                        script.exec(self);
                        self.return_enable = false;
                    }else{
                        return 1;
                    };
                },
                _ => eprintln!("Cannot read the source file: {}", &args[1]),
            }
        }
        0
    }

    pub fn return_(&mut self, _args: &mut Vec<String>) -> i32 {
        if self.return_enable {
            self.return_flag = true;
            0
        }else{
            eprintln!("Builtin return is only enabled in a function or source");
            1
        }
    }

    pub fn shopt(&mut self, args: &mut Vec<String>) -> i32 {
        if args.len() == 1 {
            self.shopts.print(true, true);
            return 0;
        }

        if args.len() == 2 && args[1] == "-s" {
            self.shopts.print(true, false);
            return 0;
        }
        if args.len() == 2 && args[1] == "-u" {
            self.shopts.print(false, true);
            return 0;
        }

        if args.len() > 2 && args[1] == "-s" {
            for opt in &mut args[2..] {
                self.shopts.set(opt, true);
            }
            return 0;
        }

        if args.len() > 2 && args[1] == "-u" {
            for opt in &mut args[2..] {
                self.shopts.set(opt, false);
            }
            return 0;
        }

        0
    }

    pub fn export(&mut self, _args: &mut Vec<String>) -> i32 {
        eprintln!("export: not implemented now");
        0
    }

    pub fn eval(&mut self, _args: &mut Vec<String>) -> i32 {
        eprintln!("export: not implemented now");
        0
    }

    pub fn glob_test(&mut self, args: &mut Vec<String>) -> i32 {
        if glob_match(&args[1].to_string(), &args[2].to_string()){
            eprintln!("MATCH!");
            0
        }else{
            eprintln!("UNMATCH!");
            1
        }
    }
}
