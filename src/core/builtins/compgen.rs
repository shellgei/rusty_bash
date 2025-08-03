//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::{path_expansion, tilde_expansion};
use crate::elements::word::{Word, WordMode};
use crate::utils;
use crate::utils::{arg, directory, glob};
use crate::{file_check, Feeder, ShellCore};
use faccess;
use faccess::PathExt;
use rev_lines::RevLines;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn compgen_f(core: &mut ShellCore, args: &mut Vec<String>, dir_only: bool) -> Vec<String> {
    if args.len() > 2 && args[2] == "--" {
        args.remove(2);
    }

    let path = match args.len() {
        2 => "".to_string(),
        _ => args[2].to_string(),
    }
    .replace("\\", "");

    let mut split: Vec<String> = path.split("/").map(|s| s.to_string()).collect();
    let key = match split.pop() {
        Some(g) => g,
        _ => return vec![],
    };

    split.push("".to_string());
    let org_dir = split.join("/");
    let mut dir = org_dir.clone();
    if dir.starts_with("~") {
        let mut feeder = Feeder::new(&dir);
        if let Ok(Some(mut w)) = Word::parse(&mut feeder, core, Some(WordMode::CompgenF)) {
            tilde_expansion::eval(&mut w, core);
            dir = w.text + &feeder.consume(feeder.len());
        }
    }

    if key.is_empty() {
        let mut files = directory::files(&dir);
        if dir_only {
            files.retain(|p| file_check::is_dir(&(dir.clone() + p)));
        }
        files.sort();
        return files.iter().map(|f| org_dir.clone() + f).collect();
    }

    let mut ans = directory::glob(&dir, &(key.clone() + "*"), &core.shopts);
    if key == "." {
        ans.append(&mut directory::glob(&dir, ".", &core.shopts));
        ans.append(&mut directory::glob(&dir, "..", &core.shopts));
    }
    ans.iter_mut().for_each(|a| {
        a.pop();
    });
    if dir_only {
        ans.retain(|p| file_check::is_dir(p));
    }
    ans.sort();
    ans.iter_mut().for_each(|e| {
        *e = e.replacen(&dir, &org_dir, 1);
    });
    ans
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
        "function" => "-A function",
        "hostname" => "-A hostname",
        "shopt" => "-A shopt",
        "stopped" => "-A stopped",
        "job" => "-j",
        "variable" => "-v",
        a => a,
    };

    args[1] = replace.to_string();
    true
}

fn command_list(target: &String, core: &mut ShellCore) -> Vec<String> {
    let mut comlist = HashSet::new();
    for path in core.db.get_param("PATH").unwrap_or_default().split(":") {
        /* /mnt is ellimimated from PATH on WSL because it contains all files in Windows.*/
        if utils::is_wsl() && path.starts_with("/mnt") && !path.ends_with("WINDOWS") {
            // We want to use explorer.exe
            continue;
        }

        for command in directory::files(path).iter() {
            if !Path::new(&(path.to_owned() + "/" + command)).executable() {
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

pub fn compgen(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() <= 1 {
        eprintln!("sush: {}: still unsupported", &args[0]);
        return 1;
    }
    let mut args = arg::dissolve_options(&args.to_vec());
    let exclude = arg::consume_with_next_arg("-X", &mut args); //TODO: implement X pattern
    let prefix = arg::consume_with_next_arg("-P", &mut args);
    let suffix = arg::consume_with_next_arg("-S", &mut args);

    replace_args_compgen(&mut args);

    let mut ans = match args[1].as_str() {
        "-a" => compgen_a(core, &args),
        "-b" => compgen_b(core, &args),
        "-c" => compgen_c(core, &mut args),
        "-d" => compgen_d(core, &mut args),
        "-f" => compgen_f(core, &mut args, false),
        "-h" => compgen_h(core, &mut args), //history (sush original)
        "-j" => compgen_j(core, &args),
        "-o" => compgen_o(core, &args),
        "-u" => compgen_u(core, &args),
        "-v" => compgen_v(core, &args),
        "-A function" => compgen_function(core, &args),
        "-A hostname" => compgen_hostname(core, &mut args),
        "-A shopt" => compgen_shopt(core, &args),
        "-A stopped" => compgen_stopped(core, &args),
        "-W" => {
            if args.len() < 2 {
                eprintln!("sush: compgen: -W: option requires an argument");
                return 2;
            }
            compgen_large_w(core, &args)
        }
        "-G" => {
            if args.len() < 2 {
                eprintln!("sush: compgen: -G: option requires an argument");
                return 2;
            }
            compgen_large_g(core, &args)
        }
        _ => {
            eprintln!("sush: compgen: {}: invalid option", &args[1]);
            return 2;
        }
    };

    if let Some(pattern) = exclude {
        let extglob = core.shopts.query("extglob");
        ans.retain(|a| !glob::parse_and_compare(a, &pattern, extglob));
    }

    if let Some(p) = prefix {
        for a in ans.iter_mut() {
            *a = p.clone() + a;
        }
    }
    if let Some(s) = suffix {
        for a in ans.iter_mut() {
            *a = a.to_owned() + &s.clone();
        }
    }

    ans.iter().for_each(|a| println!("{}", &a));
    match ans.is_empty() {
        true => 1,
        false => 0,
    }
}

fn get_head(args: &[String], pos: usize) -> String {
    if args.len() > pos && args[pos] != "--" {
        args[pos].clone()
    } else if args.len() > pos + 1 {
        args[pos + 1].clone()
    } else {
        "".to_string()
    }
}

fn drop_unmatch(args: &[String], pos: usize, list: &mut Vec<String>) {
    let head = get_head(args, pos);
    if !head.is_empty() {
        list.retain(|s| s.starts_with(&head));
    }
}

pub fn compgen_a(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut commands = vec![];

    let mut aliases: Vec<String> = core.aliases.keys().cloned().collect();
    commands.append(&mut aliases);

    let head = get_head(args, 2);
    if !head.is_empty() {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

pub fn compgen_b(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut commands = vec![];
    let mut builtins: Vec<String> = core.builtins.keys().cloned().collect();
    commands.append(&mut builtins);

    let head = get_head(args, 2);
    if !head.is_empty() {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

pub fn compgen_c(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut commands = vec![];
    if args.len() > 2 {
        commands.extend(compgen_f(core, args, false));
    }
    commands.retain(|p| Path::new(p).executable() || file_check::is_dir(p));

    let mut aliases: Vec<String> = core.aliases.keys().cloned().collect();
    commands.append(&mut aliases);
    let mut builtins: Vec<String> = core.builtins.keys().cloned().collect();
    commands.append(&mut builtins);
    let mut functions: Vec<String> = core.db.functions.keys().cloned().collect();
    commands.append(&mut functions);

    let head = get_head(args, 2);
    if !head.is_empty() {
        commands.retain(|a| a.starts_with(&head));
    }
    let mut command_in_paths = command_list(&head, core);
    commands.append(&mut command_in_paths);
    commands
}

fn compgen_d(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    compgen_f(core, args, true)
}

pub fn compgen_h(core: &mut ShellCore, _: &mut Vec<String>) -> Vec<String> {
    let len = core.history.len();
    if len >= 10 {
        return core.history[0..10].to_vec();
    }

    let mut ans = core.history.to_vec();

    if let Ok(hist_file) = File::open(core.db.get_param("HISTFILE").unwrap_or_default()) {
        for h in RevLines::new(BufReader::new(hist_file)) {
            if let Ok(s) = h {
                ans.push(s)
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

pub fn compgen_v(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut commands = vec![];

    let mut aliases: Vec<String> = core.aliases.keys().cloned().collect();
    commands.append(&mut aliases);
    let mut functions: Vec<String> = core.db.functions.keys().cloned().collect();
    commands.append(&mut functions);
    let mut vars: Vec<String> = core.db.get_keys();
    commands.append(&mut vars);

    let head = get_head(args, 2);
    if !head.is_empty() {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

pub fn compgen_o(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut commands = vec![];

    let mut options: Vec<String> = core.options.get_keys();
    commands.append(&mut options);

    let head = get_head(args, 2);
    if !head.is_empty() {
        commands.retain(|a| a.starts_with(&head));
    }
    commands
}

fn compgen_large_g(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let glob = args[2].to_string();
    path_expansion::expand(&glob, &core.shopts)
}

fn compgen_large_w(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut ans: Vec<String> = vec![];
    let mut words = args[2].to_string();

    if words.starts_with("$") {
        if let Ok(value) = core.db.get_param(&args[2][1..]) {
            words = value;
        }
    }

    let mut feeder = Feeder::new(&words);
    while !feeder.is_empty() {
        match Word::parse(&mut feeder, core, None) {
            Ok(Some(mut w)) => {
                if let Ok(mut v) = w.eval(core) {
                    ans.append(&mut v);
                }
            }
            _ => {
                let len = feeder.scanner_multiline_blank(core);
                feeder.consume(len);
            }
        }
    }

    drop_unmatch(args, 3, &mut ans);
    ans
}

pub fn compgen_u(_: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut ans = vec![];

    if let Ok(f) = File::open("/etc/passwd") {
        for line in BufReader::new(f).lines() {
            match line {
                Ok(line) => {
                    let splits: Vec<&str> = line.split(':').collect();
                    ans.push(splits[0].to_string());
                }
                _ => return vec![],
            }
        }
    }

    drop_unmatch(args, 2, &mut ans);
    ans
}

pub fn compgen_shopt(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut ans = core.shopts.get_keys();
    drop_unmatch(args, 2, &mut ans);
    ans
}

pub fn compgen_function(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut ans = core.db.functions.keys().cloned().collect();
    drop_unmatch(args, 2, &mut ans);
    ans
}

pub fn compgen_hostname(_: &mut ShellCore, _: &mut Vec<String>) -> Vec<String> {
    //TODO: Implement!
    vec![]
}

pub fn compgen_stopped(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut ans = vec![];

    for job in &core.job_table {
        if job.display_status == "Stopped" {
            ans.push(job.text.split(" ").next().unwrap().to_string());
        }
    }

    drop_unmatch(args, 2, &mut ans);
    ans
}

pub fn compgen_j(core: &mut ShellCore, args: &[String]) -> Vec<String> {
    let mut ans = vec![];

    for job in &core.job_table {
        ans.push(job.text.split(" ").next().unwrap().to_string());
    }

    drop_unmatch(args, 2, &mut ans);
    ans
}
