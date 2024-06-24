//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::DirEntry;
use std::path::Path;
use super::glob;

pub fn files(org_dir_string: &str) -> Vec<String> {
    let readdir = match org_dir_string {
        ""   => Path::new(".").read_dir(),
        dir  => Path::new(dir).read_dir(),
    };

    let to_str = |p: &DirEntry| p.file_name().to_string_lossy().to_string();

    match readdir {
        Ok(rd) => rd.map(|e| to_str(&e.unwrap()) ).collect(),
        _      => vec![],
    }
}

pub fn glob(dir_str: &str, glob_str: &str) -> Vec<String> {
    if glob_str == "" || glob_str == "." || glob_str == ".." {
        return vec![dir_str.to_string() + glob_str + "/"];
    }

    let mut fs = files(dir_str);
    fs.append( &mut vec![".".to_string(), "..".to_string()] );

    let match_condition = |f: &String| {
        ( ! f.starts_with(".") || glob_str.starts_with(".") )
        && glob::compare(f, glob_str) 
    };

    fs.iter()
      .filter(|f| match_condition(f) )
      .map(|f| dir_str.to_owned() + &f + "/")
      .collect()
}
