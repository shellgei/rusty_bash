//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use glob::glob;
use crate::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use crate::ShellCore;

pub fn chars_to_string(chars: &Vec<char>) -> String {
    chars.iter().collect::<String>()
}

