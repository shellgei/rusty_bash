//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use regex::Regex;

pub fn compare(word: &String, pattern: &str) -> bool {
    to_regex(pattern).is_match(word)
}

fn to_regex(pattern: &str) -> Regex {
    let re = Regex::new(&(r"^".to_owned() + pattern + "$")).unwrap();

    re
}
