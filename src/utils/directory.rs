//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::file;
use std::path::Path;
use super::glob;

pub fn files(dir: &str) -> Vec<String> {
    let readdir = match dir {
        "" => Path::new(".").read_dir(),
        d  => Path::new(d).read_dir(),
    };

    if ! readdir.is_ok() {
        return vec![];
    }

    readdir.unwrap()
        .map(|e| file::oss_to_name(&e.unwrap().file_name()) )
        .collect()
}

pub fn glob(dir: &str, glob: &str, extglob: bool) -> Vec<String> {
    let make_path = |file| dir.to_owned() + file + "/";

    if glob == "" || glob == "." || glob == ".." {
        return vec![make_path(glob)];
    }

    let mut fs = files(dir);
    fs.append( &mut vec![".".to_string(), "..".to_string()] );

    let compare = |file: &String| ( ! file.starts_with(".") || glob.starts_with(".") )
                            && glob::compare(file, glob, extglob);

    fs.iter().filter(|f| compare(f) ).map(|f| make_path(f) ).collect()
}
