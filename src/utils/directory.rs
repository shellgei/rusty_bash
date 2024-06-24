//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::{DirEntry, ReadDir};
use std::path::Path;
use super::glob;

fn to_readdir(dir: &str) -> Result<ReadDir, std::io::Error> {
    match dir {
        "" => Path::new(".").read_dir(),
        _  => Path::new(dir).read_dir(),
    }
}

fn dentry_to_string(p: &DirEntry) -> String {
    p.file_name().to_string_lossy().to_string()
}

pub fn files(org_dir_string: &str) -> Vec<String> {
    let readdir = match to_readdir(org_dir_string) {
        Ok(rd) => rd,
        _      => return vec![],
    };

    readdir.map(|e| dentry_to_string(&e.unwrap()) ).collect()
}

pub fn glob(dir_str: &str, glob_str: &str) -> Vec<String> {
    if glob_str == "" || glob_str == "." || glob_str == ".." {
        return vec![dir_str.to_string() + glob_str + "/"];
    }

    let readdir = match to_readdir(dir_str) {
        Ok(rd) => rd,
        _      => return vec![],
    };

    let mut files = readdir.map(|e| dentry_to_string(&e.unwrap()) )
                           .collect::<Vec<String>>();
    files.append( &mut vec![".".to_string(), "..".to_string()] );

    let match_condition = |f: &String| {
        ( ! f.starts_with(".") || glob_str.starts_with(".") )
        && glob::compare(f, glob_str) 
    };

    files.iter()
        .filter(|f| match_condition(f) )
        .map(|f| dir_str.to_owned() + &f + "/").collect()
}
