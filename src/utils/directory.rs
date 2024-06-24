//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::fs;
use std::path::Path;
use super::glob;

pub fn files(org_dir_string: &str) -> Vec<String> {
    let dir = match org_dir_string {
        ""  => ".",
        org => org, 
    };

    if ! Path::new(dir).is_dir() {
        return vec![];
    }
    let readdir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        _      => return vec![],
    };

    let mut files = vec![];
    for entry in readdir {
        if let Ok(f) = entry {
            files.push(f.file_name().to_string_lossy().to_string());
        } 
    }
    files
}

pub fn glob(org_dir_string: &str, glob_for_dir: &str) -> Vec<String> {
    let mut ans = vec![];
    if glob_for_dir == "" || glob_for_dir == "." || glob_for_dir == ".." {
        return vec![org_dir_string.to_string() + glob_for_dir + "/"];
    }

    let dir = match org_dir_string {
        ""  => ".",
        org => org, 
    };

    if ! Path::new(dir).is_dir() {
        return vec![];
    }
    let readdir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        _      => return vec![],
    };

    let mut files = vec![".".to_string(), "..".to_string()];
    for entry in readdir {
        if let Ok(f) = entry {
            files.push( f.file_name().to_string_lossy().to_string() );
        } 
    }
    for f in files {
        if f.starts_with(".") && ! glob_for_dir.starts_with(".") {
            continue;
        }

        if glob::compare(&f, glob_for_dir) {
            ans.push(org_dir_string.to_owned() + &f + "/");
        }
    }

    ans
}
