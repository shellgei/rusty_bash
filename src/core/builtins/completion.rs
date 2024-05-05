//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::word::Word;
use faccess;
use faccess::PathExt;
use std::collections::HashSet;
use std::path::PathBuf;
use std::path::Path;
use glob;
use glob::{GlobError, MatchOptions};

fn expand(path: &str) -> Vec<String> {
    let opts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    match glob::glob_with(&path, opts) {
        Ok(ps) => ps.map(|p| to_str(&p)).filter(|s| s != "").collect(),
        _ => vec![],
    }
}

fn to_str(path :&Result<PathBuf, GlobError>) -> String {
    match path {
        Ok(p) => {
            let mut s = p.to_string_lossy().to_string();
            if p.is_dir() && s.chars().last() != Some('/') {
                s.push('/');
            }
            s
        },
        _ => "".to_string(),
    }
}

fn get_paths(core: &mut ShellCore, args: &mut Vec<String>) -> Vec<String> {
    let mut path = match args.len() {
        2 => "*".to_string(),
        _ => {
            match args[2].as_str() {
                "--" => args[3].to_string() + "*",
                _ => args[2].to_string() + "*"
            }
        },
    };

    if path.starts_with("~/") {
        let home = core.data.get_param_ref("HOME").to_string() + "/";
        path = path.replace("~/", &home);
    }

    let mut paths = expand(&path);
    paths.iter_mut().for_each(|p| if p.ends_with("/") { p.pop(); });
    paths
}

fn replace_args(args: &mut Vec<String>) -> bool {
    if args.len() < 3 || args[1] != "-A" {
        return true;
    }

    args.remove(1);
    let replace = match args[1].as_str() {
        "command" => "-c",
        "file" => "-f",
        "directory" => "-d",
        a => a,
    };

    args[1] = replace.to_string();
    true
}

fn command_list(target: &String, core: &mut ShellCore) -> Vec<String> {
    let mut comlist = HashSet::new();
    for path in core.data.get_param_ref("PATH").to_string().split(":") {
        for file in expand(&(path.to_string() + "/*")) {
            if ! Path::new(&file).executable() {
                continue;
            }

            let command = file.split("/").last().map(|s| s.to_string()).unwrap();
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
        return 0;
    }

    replace_args(args);

    match args[1].as_str() {
        "-c" => compgen_c(core, args),
        "-d" => compgen_d(core, args),
        "-f" => compgen_f(core, args),
        "-W" => compgen_large_w(core, args),
        _ => {
            eprintln!("sush: compgen: {}: invalid option", &args[1]);
            return 2;
        },
    }
}

fn compgen_c(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut commands = vec![];
    if args.len() > 2 {
        commands.extend(get_paths(core, args));
    }
    commands.retain(|p| Path::new(p).executable());

    let mut aliases: Vec<String> = core.data.aliases.clone().into_keys().collect();
    commands.append(&mut aliases);
    let mut builtins: Vec<String> = core.builtins.clone().into_keys().collect();
    commands.append(&mut builtins);
    let mut functions: Vec<String> = core.data.functions.clone().into_keys().collect();
    commands.append(&mut functions);

    let head = if args.len() > 2 && args[2] != "--" {
        args[2].clone()
    }else if args.len() > 3 {
        args[3].clone()
    }else{
        "".to_string()
    };

    commands.retain(|a| a.starts_with(&head));
    let mut command_in_paths = command_list(&head, core);
    commands.append(&mut command_in_paths);

    commands.iter().for_each(|a| println!("{}", &a));
    0
}

fn compgen_d(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut paths = get_paths(core, args);
    paths.retain(|p| Path::new(p).is_dir());
    paths.iter().for_each(|a| println!("{}", &a));
    0
}

fn compgen_f(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let paths = get_paths(core, args);
    paths.iter().for_each(|a| println!("{}", a));
    0
}


fn compgen_large_w(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        eprintln!("sush: compgen: -W: option requires an argument");
        return 2;
    }

    let mut ans: Vec<String> = vec![];
    let mut feeder = Feeder::new();
    feeder.add_line(args[2].to_string());
    while feeder.len() != 0 {
        match Word::parse(&mut feeder, core) {
            Some(mut w) => {
                w.unquote();
                ans.push(w.text)
            },
            _ => {
                let len = feeder.scanner_multiline_blank(core);
                feeder.consume(len);
            },
        }
    }

    if args.len() > 3 && args[3] != "--" {
        ans.retain(|a| a.starts_with(&args[3]));
    }else if args.len() > 4 {
        ans.retain(|a| a.starts_with(&args[4]));
    }

    ans.iter().for_each(|a| println!("{}", a));
    0
}
