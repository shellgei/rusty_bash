//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::DirEntry;
use std::path::Path;
use super::glob;
use super::file_check;

pub fn files(dir: &str) -> Vec<String> {
    let dir = if dir == "" {"."}else{dir};

    let entries = match Path::new(dir).read_dir() {
        Ok(es) => es,
        Err(_) => return vec![],
    };

    let f = |e: DirEntry| e.file_name()
               .to_string_lossy().to_string();

    entries.map(|e| f(e.unwrap()) ).collect()
}

pub fn glob(dir: &str, pattern: &str, extglob: bool, dotglob: bool) -> Vec<String> {
    let make_path = |f: &str| dir.to_owned() + f + "/";

    if ["", ".", ".."].contains(&pattern) {
        let path = make_path(pattern);
        match file_check::exists(&path) {
            true  => return vec![path],
            false => return vec![],
        }
    }

    let pat = glob::parse(pattern, extglob);
    files(dir).iter()
        .filter(|f| !f.starts_with(".") || pattern.starts_with(".") || dotglob )
        .filter(|f| glob::compare(f, &pat) )
        .map(|f| make_path(&f) ).collect()
}
