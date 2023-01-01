//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::process;
use std::{io,fs,env};
use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Write, BufReader, BufRead};
use crate::bash_glob::glob_match;
use super::job::Job;
use crate::elements::command::CommandType;
use nix::sys::signal;
use nix::sys::signal::Signal;

use crate::Script;
use crate::ShellCore;
use crate::Feeder;

pub fn set_builtins(core: &mut ShellCore){
    core.builtins.insert(".".to_string(), source);
    core.builtins.insert(":".to_string(), true_);
    core.builtins.insert("alias".to_string(), alias);
    core.builtins.insert("builtin".to_string(), builtin);
    core.builtins.insert("bg".to_string(), bg);
    core.builtins.insert("cd".to_string(), cd);
    core.builtins.insert("eval".to_string(), eval);
    core.builtins.insert("exit".to_string(), exit);
    core.builtins.insert("export".to_string(), export);
    core.builtins.insert("false".to_string(), false_);
    core.builtins.insert("fg".to_string(), fg);
    core.builtins.insert("history".to_string(), history);
    core.builtins.insert("jobs".to_string(), jobs);
    core.builtins.insert("pwd".to_string(), pwd);
    core.builtins.insert("set".to_string(), set);
    core.builtins.insert("shift".to_string(), shift);
    core.builtins.insert("true".to_string(), true_);
    core.builtins.insert("read".to_string(), read);
    core.builtins.insert("return".to_string(), return_);
    core.builtins.insert("shopt".to_string(), shopt);
    core.builtins.insert("source".to_string(), source);
    core.builtins.insert("wait".to_string(), wait);

    core.builtins.insert("glob_test".to_string(), glob_test);
}

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
            eprintln!("exit: {}: numeric wordument required", args[1]);
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

pub fn bg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    fn bg_core (job: &mut Job) {
        job.status = 'R';
        println!("{}", &job.status_string());
        for p in &job.async_pids {
            signal::kill(*p, Signal::SIGCONT).unwrap();
        }
    }

    if args.len() < 2 {
        for j in 1..core.jobs.len() {
            if core.jobs[j].mark == '+' {
                bg_core(&mut core.jobs[j]);
            }
        }
        return 0;
    }

    args[1] = args[1].trim_start_matches("%").to_string();
    let job_no = if let Ok(n) = args[1].parse::<usize>() {
        n
    }else{
        eprintln!("bash: bg: {}: no such job", args[1]);
        return 1;
    };

    let status = core.jobs[job_no].status;

    if job_no >= core.jobs.len() || status == 'D' || status == 'I' {
        eprintln!("bash: bg: {}: no such job", job_no);
        return 1;
    }else if status == 'R' {
        eprintln!("bash: bg: job {} already in background", job_no);
        return 0;
    }

    bg_core(&mut core.jobs[job_no]);
    0
}

pub fn fg(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        for j in 1..core.jobs.len() {
            if core.jobs[j].mark == '+' {
                core.jobs[j].status = 'F';
       //         core.jobs[0] = core.jobs[j].clone();
       //         core.jobs[0].status = 'R';

                eprint!("{}", core.jobs[j].text);
                for p in &core.jobs[j].async_pids {
                    signal::kill(*p, Signal::SIGCONT).unwrap();
                }
                core.wait_job(j);
            }
        }
        return 0;
    }
    0
}

pub fn shift(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let num = if args.len() == 2 {
        if let Ok(n) = args[1].parse::<usize>() {
            n
        }else{
            eprintln!("bash: shift: {}: numeric wordument required", args[1]);
            return 1;
        }
    }else if args.len() == 1 {
        1
    }else{
        eprintln!("bash: shift: too many worduments");
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
        return 1;
    }
    if args.len() > 2 {
        eprintln!("{}", "bash: cd: too many arguments");
        return 1;
    }


    if args.len() == 1 { //only "cd"
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

    let wordnum = args.len() - 1;
    if wordnum < 1 {
        return 0;
    }

    let mut token = line.trim_end().split(" ").map(|s| s.to_string()).collect::<Vec<String>>();

    let last = if wordnum < token.len() {
        token[wordnum-1..].join(" ")
    }else if wordnum == token.len() {
        token[wordnum-1].clone()
    }else{
        "".to_string()
    };

    if token.len() >= wordnum-1 {
        token.insert(wordnum-1, last.clone());
    }else{
        while token.len() < wordnum {
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
                let mut feeder = Feeder::new_from(source);
                if let Some(mut script) = Script::parse(&mut feeder, core, &CommandType::Null) {
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
    for j in 1..core.jobs.len() {
        if core.jobs[j].async_pids.len() != 0 {
            core.jobs[j].check_of_finish();
        }
    }

    for (i,j) in core.jobs.iter_mut().enumerate() {
        if i == 0 {
            continue;
        }
        j.print_status();
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

pub fn eval(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
        return 0;
    }

    let text = args[1..].join(" ");
    let mut feeder = Feeder::new_from(text);
    if let Some(mut script) = Script::parse(&mut feeder, core, &CommandType::Null) {
        script.exec(core);
    }

    core.get_var("?").parse::<i32>().unwrap()
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
        if core.jobs[i].status != 'R' && core.jobs[i].status != 'F' { 
            continue;
        }
        //core.jobs[i].is_waited = true;
        core.jobs[i].status = 'F';
        core.wait_job(i);
        core.jobs[i].status = 'D';
        eprintln!("{}", &core.jobs[i].status_string());
        core.jobs[i].status = 'I';
    }

    0
}
