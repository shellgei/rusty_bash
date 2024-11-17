//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use std::ffi::OsString;
use std::path::PathBuf;

pub fn oss_to_name(oss: &OsString) -> String {
    oss.to_string_lossy().to_string()
}

pub fn buf_to_name(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}
