//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::DirEntry;
use std::path::Path;
use super::glob;

pub fn files(dir: &str) -> Vec<String> {
    let readdir = match dir {
        "" => Path::new(".").read_dir(),
        d  => Path::new(d).read_dir(),
    };

    let to_str = |p: DirEntry| p.file_name().to_string_lossy().to_string();

    match readdir {
        Ok(rd) => rd.map(|e| to_str(e.unwrap()) ).collect(),
        _      => vec![],
    }
}

pub fn glob(dir: &str, glob: &str) -> Vec<String> {
    if glob == "" || glob == "." || glob == ".." {
        return vec![dir.to_string() + glob + "/"];
    }

    let mut fs = files(dir);
    fs.append( &mut vec![".".to_string(), "..".to_string()] );

    let make_path = |f| dir.to_owned() + f + "/";
    let compare = |f: &String| ( ! f.starts_with(".") || glob.starts_with(".") )
                            && glob::compare(f, glob);

    fs.iter().filter(|f| compare(f) ).map(|f| make_path(f) ).collect()
}
