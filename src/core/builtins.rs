//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::process;
use std::{io,fs,env};
use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Write, BufReader, BufRead};
use crate::bash_glob::glob_match;
use crate::element_list::CompoundType;

use crate::Script;
use crate::ShellCore;
use crate::Feeder;

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let home = env::var("HOME").expect("HOME is not defined");
    if let Ok(mut hist_file) = OpenOptions::new().write(true)
                               .append(true).open(home + "/.bash_history") {
        for h in &core.history {
            write!(hist_file, "{}\n", h).expect("Cannot write history");
        };
        hist_file.flush().expect("Cannot flush the history file");
    }

    if args.len() >= 2 {
        if let Ok(status) = args[1].parse::<i32>(){
            process::exit(status);
        }else{
            eprintln!("exit: {}: numeric argument required", args[1]);
            process::exit(2);
        }
    }

    if let Ok(status) = core.get_var("?").to_string().parse::<i32>(){
        process::exit(status);
    }else{
        eprintln!("Shell internal error");
        process::exit(1);
    }
}

pub fn history(_core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    let home = env::var("HOME").expect("HOME is not defined");
    if let Ok(hist_file) = OpenOptions::new().read(true).open(home + "/.bash_history") {
        let reader = BufReader::new(hist_file);
        for (i, line) in reader.lines().enumerate() {
            if let Ok(s) = line {
                println!("  {}  {}", i, s);
            }
        }
    }
    0
}


pub fn pwd(_core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    if let Some(p) = env::current_dir().expect("Cannot get current dir").to_str() {
        println!("{}", p.to_string());
        return 0;
    };

    panic!("Cannot get current dir");
}

pub fn true_(_core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    0
}

pub fn false_(_core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    1
}

pub fn shift(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let num = if args.len() == 2 {
        if let Ok(n) = args[1].parse::<usize>() {
            n
        }else{
            eprintln!("bash: shift: {}: numeric argument required", args[1]);
            return 1;
        }
    }else if args.len() == 1 {
        1
    }else{
        eprintln!("bash: shift: too many arguments");
        return 1;
    };

    if core.args.len() < num+1 {
        return 1;
    }

    for _ in 0..num {
        core.args.remove(1);
    }
    0
}

pub fn builtin(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() > 0 {
        args.remove(0);
    }
    if args.len() < 1 {
        return 0;
    }

    if let Some(func) = core.get_builtin(&args[0]) {
        func(core, args)
    }else{
        eprintln!("bash: builtin: {}: not a shell builtin", args[0]);
        1
    }
}

pub fn cd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 0 {
        eprintln!("Bug of this shell");
    }else if args.len() == 1 { //only "cd"
        let var = env::var("HOME").expect("HOME is not defined");
        args.push(var);
    }else if args.len() == 2 && args[1] == "-" { // cd -
        if let Some(old) = core.vars.get("OLDPWD") {
            args[1] = old.to_string();
        }
    };

    if let Ok(old) = env::current_dir() {
        core.set_var("OLDPWD", &old.display().to_string());
    };

    let path = Path::new(&args[1]);
    if env::set_current_dir(&path).is_ok() {
        if let Ok(full) = fs::canonicalize(path) {
            core.set_var("PWD", &full.display().to_string());
        }
        0
    }else{
        eprintln!("Not exist directory");
        1
    }
}

pub fn alias(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        for (k, v) in core.aliases.iter() {
            println!("alias {}='{}'", k, v);
        }
        return 0;
    }

    if let Some(com) = core.aliases.get(&args[1]) {
        println!("alias {}='{}'", &args[1], com);
        return 0;
    }

    let elems = args[1].split('=').collect::<Vec<&str>>();
    if elems.len() < 2 {
        eprintln!("bash: alias: {} not found", &args[1]);
        return 1;
    }

    core.aliases.insert(elems[0].to_string(), elems[1..].join("="));
    0
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
       for k in core.vars.keys() {
           println!("{}={}", k, core.vars[k]);
       }
       return 0;
    }

    core.args.clear();

    for a in args {
        core.args.push(a.to_string());
    }

    0
}

pub fn read(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
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

    for (i, a) in args.iter().enumerate() {
        if i == 0 {
            continue;
        }

        core.set_var(a, &token[i-1]);
    }

    0
}

pub fn source(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        eprintln!("usage: source filename");
        return 1;
    }

    if args.len() > 1 {
        match fs::read_to_string(&args[1]) {
            Ok(source) => {
                let mut feeder = Feeder::new_with(source);
                if let Some(mut script) = Script::parse(&mut feeder, core, &CompoundType::Null) {
                    core.return_enable = true;
                    script.exec(core);
                    core.return_enable = false;
                }else{
                    return 1;
                };
            },
            _ => eprintln!("Cannot read the source file: {}", &args[1]),
        }
    }
    0
}

pub fn return_(core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    if core.return_enable {
        core.return_flag = true;
        0
    }else{
        eprintln!("Builtin return is only enabled in a function or source");
        1
    }
}

pub fn jobs(core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    for (i,j) in core.jobs.iter().enumerate() {
        if i == 0 {
            continue;
        }
        println!("[{}] {}", i, j.clone().status_string().trim_end());
    }

    0
}

pub fn shopt(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
        core.shopts.print(true, true);
        return 0;
    }

    if args.len() == 2 && args[1] == "-s" {
        core.shopts.print(true, false);
        return 0;
    }
    if args.len() == 2 && args[1] == "-u" {
        core.shopts.print(false, true);
        return 0;
    }

    if args.len() > 2 && args[1] == "-s" {
        for opt in &mut args[2..] {
            core.shopts.set(opt, true);
        }
        return 0;
    }

    if args.len() > 2 && args[1] == "-u" {
        for opt in &mut args[2..] {
            core.shopts.set(opt, false);
        }
        return 0;
    }

    0
}

pub fn export(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 { // TODO: it should output all env vars. 
        return 1;
    }

    let key_value = args[1].split("=").map(|s| s.to_string()).collect::<Vec<String>>();

    let (key, value) = if key_value.len() == 0 {
        return 1;
    }else if key_value.len() == 1 {
        (&key_value[0], "".to_string())
    }else{
        (&key_value[0], key_value[1..].join("="))
    };

    if ! core.vars.contains_key(key) {
        env::set_var(key, value);
        return 0;
    }

    let value = core.get_var(&key);
    env::set_var(key, value);
    core.vars.remove(key);

    0
}

pub fn eval(_core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    eprintln!("eval: not implemented now");
    0
}

pub fn glob_test(_core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if glob_match(&args[1].to_string(), &args[2].to_string()){
        eprintln!("MATCH!");
        0
    }else{
        eprintln!("UNMATCH!");
        1
    }
}

pub fn wait(core: &mut ShellCore, _args: &mut Vec<String>) -> i32 {
    for i in 1..core.jobs.len() {
        core.wait_job(i);
    }

    0
}
