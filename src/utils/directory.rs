//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::DirEntry;
use std::path::Path;

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

pub fn glob(dir: &str, _: &str) -> Vec<String> {
    files(dir)
}
