//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::path::{Path, PathBuf, Component};

fn make_absolute_path(core: &mut ShellCore, path_str: &str) -> PathBuf {
    let path = Path::new(&path_str);
    let mut absolute = PathBuf::new();
    if path.is_relative() {
        if path.starts_with("~") { // tilde -> $HOME
            if let Some(home_dir) = core.vars.get("HOME") {
                absolute.push(PathBuf::from(home_dir));
                if path_str.len() > 1 && path_str.starts_with("~/") {
                    absolute.push(PathBuf::from(&path_str[2..]));
                } else {
                    absolute.push(PathBuf::from(&path_str[1..]));
                }
            }
        } else { // current
            if let Some(tcwd) = &core.get_current_directory() {
                absolute.push(tcwd);
                absolute.push(path);
            };
        }
    } else {
        absolute.push(path);
    }
    absolute
}

fn make_canonical_path(path: PathBuf) -> PathBuf {
    let mut canonical = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir => canonical.push(Component::RootDir),
            Component::ParentDir => { canonical.pop(); }, 
            Component::Normal(c) => canonical.push(c),
            _ => (),
        }
    }
    canonical
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.vars.insert("?".to_string(), args[1].clone());
    }
    core.exit()
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
        let var = "~".to_string();
        args.push(var);
    }else if args.len() == 2 && args[1] == "-" { // cd -
        if let Some(old) = core.vars.get("OLDPWD") {
            println!("{}", &old);
            args[1] = old.to_string();
        }
    };

    if let Some(old) = core.get_current_directory().clone() {
        core.vars.insert("OLDPWD".to_string(), old.display().to_string());
    };

    let path = make_canonical_path(make_absolute_path(core, &args[1]));
    if core.set_current_directory(&path).is_ok() {
        core.vars.insert("PWD".to_string(), path.display().to_string());
        0
    }else{
        eprintln!("Not exist directory");
        1
    }
}

pub fn pwd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut physical: bool = false;

    match args.len() {
        0 => {
            eprintln!("Bug of this shell");
            return 1;    
        },
        2 => {
            if &args[1][..1] == "-" {
                match args[1].as_str() {
                    "-P" => { physical = true }, // シンボリックリンク名を解決して表示する
                    "-L" => (), // シンボリックリンク名をそのまま表示する（bash default）
                    _ => {
                        eprintln!("{}", "sush: pwd: invalid option");
                        eprintln!("{}", "pwd: usage: pwd [-LP]");
                        return 1;
                    },
                }
            }
        },
        _ => (),
    }

    if let Some(mut path) = core.get_current_directory().clone() {
        if physical && path.is_symlink() {
            if let Ok(c) = path.canonicalize() {
                path = c;
            }
        }
        println!("{}", path.display());
        return 0;
    }
    1
}

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}

pub fn false_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    1
}
