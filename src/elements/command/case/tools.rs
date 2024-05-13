//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use regex::Regex;

pub fn compare(word: &String, pattern: &str) -> bool {
    to_regex(pattern).is_match(word)
}

fn to_regex(pattern: &str) -> Regex {
    let mut regex_str = String::new();
    let mut remaining = pattern.to_string();

    while remaining.len() > 0 {
        let len = scanner_escaped_char(&remaining);
        if len > 0 {
            let mut ans = consume(&mut remaining, len);
            ans.remove(0);
            regex_str += &ans;
            continue;
        }

        let len = scanner_char(&remaining);
        if len > 0 {
            regex_str += &consume(&mut remaining, len);
        }
    }

    let re = Regex::new(&(r"^".to_owned() + &regex_str + "$")).unwrap();

    re
}

fn scanner_escaped_char(remaining: &str) -> usize {
    if ! remaining.starts_with("\\") {
        return 0;
    }

    match remaining.chars().nth(1) {
        None    => 1,
        Some(c) => 1 + c.len_utf8(),
    }
}

fn scanner_char(remaining: &str) -> usize {
    match remaining.chars().nth(0) {
        None    => 0,
        Some(c) => c.len_utf8(),
    }
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining[cutpos..].to_string();

    cut
}
