//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::word::Word;
use faccess;
use faccess::PathExt;
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

pub fn compgen_c(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut commands = vec![];
    if args.len() > 2 {
        commands.extend(get_paths(core, args));
    }
    commands.retain(|p| Path::new(p).executable());
    commands.iter().for_each(|a| println!("{}", &a));
    0
}

pub fn compgen_d(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut paths = get_paths(core, args);
    paths.retain(|p| Path::new(p).is_dir());
    paths.iter().for_each(|a| println!("{}", &a));
    0
}

pub fn compgen_f(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let paths = get_paths(core, args);
    paths.iter().for_each(|a| println!("{}", a));
    0
}


pub fn compgen_large_w(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
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
