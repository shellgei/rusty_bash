//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

pub fn exists(name: &str) -> bool {
    fs::metadata(name).is_ok()
}

pub fn is_file(name: &str) -> bool {
    Path::new(name).is_file()
}

pub fn is_dir(name: &str) -> bool {
    Path::new(name).is_dir()
}

pub fn type_check(name: &str, tp: &str) -> bool {
    let meta = match fs::metadata(name) {
        Ok(m) => m,
        _     => return false,
    };

    match tp {
        "-b" => meta.file_type().is_block_device(),
        "-c" => meta.file_type().is_char_device(),
        _ => false,
    }
}
