//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::file_check;
use super::glob;
use crate::core::options::Options;
use std::fs::DirEntry;
use std::path::Path;

pub fn files(dir: &str) -> Vec<String> {
    let dir = if dir.is_empty() { "." } else { dir };

    let entries = match Path::new(dir).read_dir() {
        Ok(es) => es,
        Err(_) => return vec![],
    };

    let f = |e: DirEntry| e.file_name().to_string_lossy().to_string();

    entries.map(|e| f(e.unwrap())).collect()
}

fn globstar(dir: &str) -> Vec<String> {
    let dir = if dir.is_empty() || dir.ends_with("/") {
        dir
    } else {
        &(dir.to_owned() + "/")
    };
    let mut dirs = vec![dir.to_string()];
    //    let mut ans = dirs.clone();
    let mut ans = vec![];

    while !dirs.is_empty() {
        let mut tmp = vec![];
        for d in dirs {
            if file_check::is_symlink(d.trim_end_matches("/")) {
                continue;
            }
            let mut fs = files(&d);
            fs.iter_mut().for_each(|f| {
                *f = d.to_string() + f + "/";
            });
            tmp.append(&mut fs);
        }
        ans.extend(tmp.clone());
        dirs = tmp;
        dirs.sort();
        dirs.dedup();
    }

    ans.sort();
    ans.dedup();
    ans
}

pub fn glob(dir: &str, pattern: &str, shopts: &Options) -> Vec<String> {
    let make_path = |f: &str| dir.to_owned() + f + "/";

    if ["", ".", ".."].contains(&pattern)
        || (file_check::is_symlink(dir.trim_end_matches("/")) && shopts.query("globstar"))
    {
        let path = make_path(pattern);
        match file_check::exists(&path) {
            true => return vec![path],
            false => return vec![],
        }
    }

    if pattern == "**" && shopts.query("globstar") {
        return globstar(dir);
    }

    let dotglob = shopts.query("dotglob");
    let extglob = shopts.query("extglob");
    let pat = glob::parse(pattern, extglob);
    let mut ans: Vec<String> = files(dir)
        .iter()
        .filter(|f| !f.starts_with(".") || pattern.starts_with(".") || dotglob)
        .filter(|f| glob::compare(f, &pat))
        .map(|f| make_path(f))
        .collect();

    if !shopts.query("globskipdots") {
        if glob::compare(&"..".to_string(), &pat) {
            ans.push(make_path(".."));
        }
        if glob::compare(&".".to_string(), &pat) {
            ans.push(make_path("."));
        }
    }

    ans.sort();
    ans.dedup();
    ans
}
