//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::OsString;
use std::path::Path;

pub fn files(dir: &str) -> Vec<String> {
    let readdir = match dir {
        "" => Path::new(".").read_dir(),
        d  => Path::new(d).read_dir(),
    };

    if ! readdir.is_ok() {
        return vec![];
    }

    let f = |e: OsString| e.to_string_lossy().to_string();
    readdir.unwrap()
        .map(|e| f(e.unwrap().file_name()) )
        .collect()
}
