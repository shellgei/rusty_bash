//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::options::Options;
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

fn globstar(dir: &str) -> Vec<String> {
    let mut dirs = files(dir);
    if dir != "" && ! dir.ends_with("/") {
        dirs.iter_mut().for_each(|d| {*d = dir.to_string() + "/" + &d; });
    }
    let mut ans = dirs.clone();

    for d in dirs {
        if ! file_check::is_symlink(&d) {
            ans.append(&mut globstar(&d));
        }
    }

    ans
}

pub fn glob(dir: &str, pattern: &str, shopts: &Options) -> Vec<String> {
    let make_path = |f: &str| dir.to_owned() + f + "/";
    if ["", ".", ".."].contains(&pattern) 
    || file_check::is_symlink(dir.trim_end_matches("/")) {
        let path = make_path(pattern);
        match file_check::exists(&path) {
            true  => return vec![path],
            false => return vec![],
        }
    }

    if pattern == "**" && shopts.query("globstar") {
        let mut tmp = globstar(dir);
        tmp.push(dir.to_string());
        tmp.iter_mut().for_each(|d| if d != "" && ! d.ends_with("/") {*d += "/"; });
        tmp.sort();
        tmp.dedup();
        return tmp;
    }

    let dotglob = shopts.query("dotglob");
    let extglob = shopts.query("extglob");
    let pat = glob::parse(pattern, extglob);
    files(dir).iter()
        .filter(|f| !f.starts_with(".") || pattern.starts_with(".") || dotglob )
        .filter(|f| glob::compare(f, &pat) )
        .map(|f| make_path(&f) ).collect()
}
