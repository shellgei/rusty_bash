//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;
use std::process::exit;
use std::env;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;

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

pub struct ShellCore {
    pub internal_commands: HashMap<String, fn(&mut ShellCore, args: &mut Vec<String>) -> i32>,
    pub vars: HashMap<String, String>,
    pub aliases: HashMap<String, String>,
    pub history: Vec<String>,
    pub flags: Flags,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut conf = ShellCore{
            internal_commands: HashMap::new(),
            vars: HashMap::new(),
            aliases: HashMap::new(),
            history: Vec::new(),
            flags: Flags::new(),
        };

        conf.vars.insert("?".to_string(), 0.to_string());

        conf.internal_commands.insert("exit".to_string(), Self::exit);
        conf.internal_commands.insert("pwd".to_string(), Self::pwd);
        conf.internal_commands.insert("cd".to_string(), Self::cd);
        conf.internal_commands.insert("alias".to_string(), Self::alias);

        conf
    }

    pub fn get_var(&self, key: &String) -> String {
        if let Some(s) = self.vars.get(&key as &str){
            return s.to_string();
        };

        if let Ok(s) = env::var(&key) {
            return s.to_string();
        };

        "".to_string()
    }

    pub fn get_internal_command(&self, name: &String) 
        -> Option<fn(&mut ShellCore, args: &mut Vec<String>) -> i32> {
        if self.internal_commands.contains_key(name) {
            Some(self.internal_commands[name])
        }else{
            None
        }
    }
    /////////////////////////////////
    /* INTERNAL COMMANDS HEREAFTER */
    /////////////////////////////////
    pub fn exit(&mut self, _args: &mut Vec<String>) -> i32 {
        let home = env::var("HOME").expect("HOME is not defined");
        let mut hist_file = OpenOptions::new()
                                    .write(true)
                                    .append(true)
                                    .open(home + "/.bash_history")
                                    .expect("Cannot open the history file");

        for h in &self.history {
            write!(hist_file, "{}\n", h).expect("Cannot write history");
        };
        hist_file.flush().expect("Cannot flush the history file");

        exit(0);
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
        }else if args.len() == 1 {
            let var = env::var("HOME").expect("HOME is not defined");
            args.push(var);
        };

        let path = Path::new(&args[1]);
        if env::set_current_dir(&path).is_ok() {
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
}
