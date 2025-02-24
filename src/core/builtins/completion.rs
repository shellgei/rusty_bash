//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{file_check, ShellCore, Feeder};
use crate::core::{CompletionInfo, HashMap};
use crate::elements::word::Word;
use crate::elements::word::tilde_expansion;
use crate::utils;
use crate::utils::{arg, directory};
use faccess;
use faccess::PathExt;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use rev_lines::RevLines;

pub fn compgen_f(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    if args.len() > 2 && args[2] == "--" {
        args.remove(2);
    }

    let path = match args.len() {
        2 => "".to_string(),
        _ => args[2].to_string(),
    }.replace("\\", "");

    let mut split: Vec<String> = path.split("/").map(|s| s.to_string()).collect();
    let key = match split.pop() {
        Some(g) => g, 
        _       => return vec![],
    };

    split.push("".to_string());
    let org_dir = split.join("/");
    let mut dir = org_dir.clone();
    if dir.starts_with("~") {
        if let Ok(Some(mut w)) = Word::parse(&mut Feeder::new(&dir), core, true) {
            tilde_expansion::eval(&mut w, core);
            dir = w.text;
        }
    }

    if key == "" {
        let files = directory::files(&dir);
        return files.iter().map(|f| org_dir.clone() + &f).collect();
    }

    let mut ans = directory::glob(&dir, &(key.clone() + "*"), core.shopts.query("extglob"));
    if key == "." {
        ans.append(&mut directory::glob(&dir, ".", false));
        ans.append(&mut directory::glob(&dir, "..", false));
    }
    ans.iter_mut().for_each(|a| { a.pop(); } );
    ans.sort();
    ans.iter_mut().for_each(|e| {*e = e.replacen(&dir, &org_dir, 1); });
    ans
}

fn action_to_reduce_symbol(arg: &str) -> String {
    match arg {
        "file" => "f",
        "directory" => "d",
        "command" => "c",
        "alias" => "a",
        "builtin" => "b",
        "keyword" => "k",
        "variable" => "v",
        "export" => "e",
        "setopt" => "o",
        "job" => "j",
        "service" => "s",
        "user" => "u",
        "group" => "g",
        _ => "",
    }.to_string()
}

fn replace_args_compgen(args: &mut Vec<String>) -> bool {
    if args.len() < 3 || args[1] != "-A" {
        return true;
    }

    args.remove(1);
    let replace = match args[1].as_str() {
        "command" => "-c",
        "directory" => "-d",
        "file" => "-f",
        "user" => "-u",
        "setopt" => "-o",
        "stopped" => "-A stopped",
        "job" => "-j",
        a => a,
    };

    args[1] = replace.to_string();
    true
}

fn command_list(target: &String, core: &mut ShellCore) -> Vec<String> {

    let mut comlist = HashSet::new();
    for path in core.db.get_param("PATH").unwrap_or(String::new()).to_string().split(":") {
        if utils::is_wsl() && path.starts_with("/mnt") {
            continue;
        }

        for command in directory::files(path).iter() {
            if ! Path::new(&(path.to_owned() + "/" + command)).executable() {
                continue;
            }

            if command.starts_with(target) {
                comlist.insert(command.clone());
            }
        }
    }
    let mut ans: Vec<String> = comlist.iter().map(|c| c.to_string()).collect();
    ans.sort();
    ans
}

pub fn compgen(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        eprintln!("sush: {}: still unsupported", &args[0]);
        return 1;
    }
    let mut args = arg::dissolve_options(args);
    let _  = arg::consume_with_next_arg("-X", &mut args); //TODO: implement X pattern
    let prefix = arg::consume_with_next_arg("-P", &mut args);

    replace_args_compgen(&mut args);

    let mut ans = match args[1].as_str() {
        "-a" => compgen_a(core, &mut args),
        "-b" => compgen_b(core, &mut args),
        "-c" => compgen_c(core, &mut args),
        "-d" => compgen_d(core, &mut args),
        "-f" => compgen_f(core, &mut args),
        "-h" => compgen_h(core, &mut args), //history (sush original)
        "-j" => compgen_j(core, &mut args),
        "-u" => compgen_u(core, &mut args),
        "-v" => compgen_v(core, &mut args),
        "-A stopped" => compgen_stopped(core, &mut args),
        "-W" => {
            if args.len() < 2 {
                eprintln!("sush: compgen: -W: option requires an argument");
                return 2;
            }
            compgen_large_w(core, &mut args)
        },
        _ => {
            eprintln!("sush: compgen: {}: invalid option", &args[1]);
            return 2;
        },
    };

    if let Some(p) = prefix {
        for a in ans.iter_mut() {
            *a = p.clone() + a;
        }
    }

    ans.iter().for_each(|a| println!("{}", &a));
    0
}

fn get_head(args: &mut Vec<String>, pos: usize) -> String {
    if args.len() > pos && args[pos] != "--" {
        args[pos].clone()
    }else if args.len() > pos+1 {
        args[pos+1].clone()
    }else{
        "".to_string()
    }
}

fn drop_unmatch(args: &mut Vec<String>, pos: usize, list: &mut Vec<String>) {
    let head = get_head(args, pos);
    if head != "" {
        list.retain(|s| s.starts_with(&head));
    }
}

pub fn compgen_a(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut commands = vec![];

    let mut aliases: Vec<String> = core.aliases.clone().into_keys().collect();
    commands.append(&mut aliases);

    let head = get_head(args, 2);
    if head != "" {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

pub fn compgen_b(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut commands = vec![];
    let mut builtins: Vec<String> = core.builtins.clone().into_keys().collect();
    commands.append(&mut builtins);

    let head = get_head(args, 2);
    if head != "" {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

pub fn compgen_c(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut commands = vec![];
    if args.len() > 2 {
        commands.extend(compgen_f(core, args));
    }
    commands.retain(|p| Path::new(p).executable() || file_check::is_dir(p));

    let mut aliases: Vec<String> = core.aliases.clone().into_keys().collect();
    commands.append(&mut aliases);
    let mut builtins: Vec<String> = core.builtins.clone().into_keys().collect();
    commands.append(&mut builtins);
    let mut functions: Vec<String> = core.db.functions.clone().into_keys().collect();
    commands.append(&mut functions);

    let head = get_head(args, 2);
    if head != "" {
        commands.retain(|a| a.starts_with(&head));
    }
    let mut command_in_paths = command_list(&head, core);
    commands.append(&mut command_in_paths);
    commands
}

fn compgen_d(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut paths = compgen_f(core, args);
    paths.retain(|p| file_check::is_dir(&p));
    paths
}

pub fn compgen_h(core: &mut ShellCore, _: &mut Vec<String>) -> Vec<String> {
    let len = core.history.len();
    if len >= 10 {
        return core.history[0..10].to_vec();
    }

    let mut ans = core.history.to_vec();

    if let Ok(hist_file) = File::open(core.db.get_param("HISTFILE").unwrap_or(String::new())){
        for h in RevLines::new(BufReader::new(hist_file)) {
            match h {
                Ok(s) => ans.push(s),
                _     => {},
            }

            if ans.len() >= 10 {
                return ans;
            }
        }
    }

    while ans.len() < 10 {
        ans.push("echo Hello World".to_string());
    }
    ans
}

pub fn compgen_v(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut commands = vec![];

    let mut aliases: Vec<String> = core.aliases.clone().into_keys().collect();
    commands.append(&mut aliases);
    let mut functions: Vec<String> = core.db.functions.clone().into_keys().collect();
    commands.append(&mut functions);
    let mut vars: Vec<String> = core.db.get_keys();
    commands.append(&mut vars);

    let head = get_head(args, 2);
    if head != "" {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

pub fn compgen_o(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut commands = vec![];

    let mut options: Vec<String> = core.options.get_keys();
    commands.append(&mut options);

    let head = get_head(args, 2);
    if head != "" {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

fn compgen_large_w(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut ans: Vec<String> = vec![];
    let mut words = args[2].to_string();

    if words.starts_with("$") {
        if let Ok(value) = core.db.get_param(&args[2][1..]) {
            words = value;
        }
    }

    let mut feeder = Feeder::new(&words);
    while feeder.len() != 0 {
        match Word::parse(&mut feeder, core, false) {
            Ok(Some(mut w)) => {
                w.make_unquoted_word();
                ans.push(w.text)
            },
            _ => {
                let len = feeder.scanner_multiline_blank(core);
                feeder.consume(len);
            },
        }
    }

    drop_unmatch(args, 3, &mut ans);
    ans
}

pub fn compgen_u(_: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut ans = vec![];

    if let Ok(f) = File::open("/etc/passwd") {
        for line in BufReader::new(f).lines() {
            match line {
                Ok(line) => {
                    let splits: Vec<&str> = line.split(':').collect();
                    ans.push(splits[0].to_string());
                },
                _ => return vec![],
            }
        }
    }

    drop_unmatch(args, 2, &mut ans);
    ans
}

pub fn compgen_stopped(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut ans = vec![];

    for job in &core.job_table {
        if job.display_status == "Stopped" {
            ans.push(job.text.split(" ").nth(0).unwrap().to_string());
        }
    }

    drop_unmatch(args, 2, &mut ans);
    ans
}

pub fn compgen_j(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut ans = vec![];

    for job in &core.job_table {
        ans.push(job.text.split(" ").nth(0).unwrap().to_string());
    }

    drop_unmatch(args, 2, &mut ans);
    ans
}

fn opt_to_action(arg: &str) -> String {
    match arg {
        "-a" => "alias",
        "-b" => "builtin",
        "-c" => "command",
        "-j" => "job",
        "-o" => "setopt",
        "-u" => "user",
        "-v" => "variable",
        _ => "",
    }.to_string()
}

fn print_complete(core: &mut ShellCore) -> i32 {
    if core.default_completion_functions != "" {
        println!("complete -F {} -D", &core.default_completion_functions);
    }

    for (name, info) in &core.completion_info {
        if info.function != "" {
            print!("complete -F {} ", &info.function);
        }else if info.action != "" {
            let symbol = action_to_reduce_symbol(&info.action);

            if symbol == "" {
                print!("complete -A {} ", &info.action);
            }else{
                print!("complete -{} ", &symbol);
            }

            if info.options.contains_key("-P") {
                print!("-P '{}' ", &info.options["-P"]);
            }
            if info.options.contains_key("-S") {
                print!("-S '{}' ", &info.options["-S"]);
            }
        }else{
            print!("complete ");
        }
        println!("{}", &name); 
    }
    0
}

pub fn complete(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return print_complete(core);
    }

    let mut o_options = vec![];
    let mut args = arg::dissolve_options(args);
    while let Some(v) = arg::consume_with_next_arg("-o", &mut args) {
        o_options.push(v);
    }

    let mut options = HashMap::new();
    let prefix = arg::consume_with_next_arg("-P", &mut args);
    if prefix != None {
        options.insert("-P".to_string(), prefix.unwrap().clone());
    }
    let suffix = arg::consume_with_next_arg("-S", &mut args);
    if suffix != None {
        options.insert("-S".to_string(), suffix.unwrap().clone());
    }

    let action = opt_to_action(&args[1]);
    if action != "" {
        for command in &args[2..] {
            if ! core.completion_info.contains_key(command) {
                core.completion_info.insert(command.clone(), CompletionInfo::default());
            }
    
            let info = &mut core.completion_info.get_mut(command).unwrap();
            info.action = action.clone();
            info.options = options.clone();
        }
        return 0;
    }

    if args.len() > 3 && args[1] == "-A" {
        for command in &args[3..] {
            if ! core.completion_info.contains_key(command) {
                core.completion_info.insert(command.clone(), CompletionInfo::default());
            }
    
            let info = &mut core.completion_info.get_mut(command).unwrap();
            info.action = args[2].clone();
            info.options = options.clone();
        }

        return 0;
    }
    //complete -F _comp_cmd_gpg2 -o default gpg2

    // completion functions
    if args.len() > 3 && args[1] == "-D" && args[2] == "-F" {
        core.default_completion_functions = args[3].clone();
        return 0;
    }

    if args.len() > 3 && args[1] == "-F" {
        let func = args[2].clone();
        for command in &args[3..] {
            if ! core.completion_info.contains_key(command) {
                core.completion_info.insert(command.clone(), CompletionInfo::default());
            }
    
            let info = &mut core.completion_info.get_mut(command).unwrap();
            info.function = func.clone();
            info.o_options = o_options.clone();
        }

        return 0;
    }

    eprintln!("sush: {} {}: still unsupported", &args[0], &args[1]);
    1
}

fn compopt_set_current(core: &mut ShellCore, plus: &Vec<String>, minus: &Vec<String>) -> i32 {
    let com = &core.current_completion_target;
    dbg!("{:?}", &com);

    if com == "" {
        return 1;
    }

    let info = match core.completion_info.get_mut(com) {
        Some(i) => i,
        None => return 1,
    };

    for opt in minus { //add
        if ! info.o_options.contains(opt) {
            info.o_options.push(opt.to_string());
        }
    }

    for opt in plus { //remove
        info.o_options.retain(|e| e != opt);
    }

    0
}

fn compopt_print(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let optlist = vec!["bashdefault", "default",
                       "dirnames", "filenames", "noquote",
                       "nosort", "nospace", "plusdirs"];
    let optlist: Vec<String> = optlist.iter().map(|s| s.to_string()).collect();

    let com = args[1].clone();
    if core.completion_info.contains_key(&com) {
        let info = &core.completion_info.get_mut(&com).unwrap();

        print!("compopt ");
        for opt in &optlist {
            match info.o_options.contains(opt) {
                true  => print!("-o {} ", opt), 
                false => print!("+o {} ", opt), 
            }
        }
        println!("{}", &com);
    }else{
        eprintln!("sush: compopt: {}: no completion specification", &args[1]);
        return 1;
    }

    0
}

pub fn compopt(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        dbg!("{:?}", &core.completion_info);
        return 1;
    }

    if ! args[1].starts_with("-") && ! args[1].starts_with("+") {
        return compopt_print(core, args);
    }

    let mut minus = vec![];
    loop {
        let opt = arg::consume_with_next_arg("-o", args);
        if opt.is_none() {
            break;
        }
        minus.push(opt.unwrap());
    }

    let mut plus = vec![];
    loop {
        let opt = arg::consume_with_next_arg("+o", args);
        if opt.is_none() {
            break;
        }
        plus.push(opt.unwrap());
    }

    if args.len() == 1 {
        return compopt_set_current(core, &plus, &minus);
    }

    /*
    let com = args.last().unwrap();
    if com == "-E" || com == "-D" {
        eprintln!("not supported yet");
        return 1;
    }

    dbg!("{:?}", &com);
    */

    0
}
