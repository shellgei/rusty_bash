//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::file_check;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

pub fn oss_to_name(oss: &OsString) -> String {
    oss.to_string_lossy().to_string()
}

pub fn buf_to_name(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

pub fn search_command(command: &str) -> Option<String> {
    let paths = env::var_os("PATH");
    if paths.is_none() {
        return None;
    }

    for path in env::split_paths(&paths.unwrap()) {
        let compath = buf_to_name(&path) + "/" + command;
        if file_check::exists(&compath) {
            return Some(compath);
        }
    }

    None
}
