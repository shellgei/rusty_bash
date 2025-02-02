//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use super::glob;

pub fn files(dir: &str) -> Vec<String> {
    let dir = if dir.is_empty() {"."}else{dir};

    let entries = match Path::new(dir).read_dir() {
        Ok(es) => es,
        Err(_) => return vec![],
    };

    let f = |e: DirEntry| e.file_name()
               .to_string_lossy().to_string();

    entries.map(|e| f(e.unwrap()) ).collect()
}

pub fn glob(dir: &str, pattern: &str) -> Vec<String> {
    let make_path = |f: &str| dir.to_owned() + f + "/";

    if pattern == "" {
        let path = make_path(pattern);
        match fs::metadata(&path).is_ok() {
            true  => return vec![path],
            false => return vec![],
        }
    }

    let pat = glob::parse(pattern);
    files(dir).iter()
        .filter(|f| glob::compare(f, &pat) )
        .map(|f| make_path(f) )
        .collect()
}
