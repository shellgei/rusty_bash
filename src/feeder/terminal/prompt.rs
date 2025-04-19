//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::Path;
use crate::utils::{file, file_check};
use nix::unistd::{getuid, gethostname, getcwd, User};
use std::char;

pub fn oct_to_hex_in_str(from: &str) -> String {
    fn oct_string(s: &str) -> bool {
        if !s.starts_with('\\') { return false; }
        s.chars().skip(1).take(3).all(|c| c >= '0' && c <= '9')
    }
    let mut i = 0;
    let mut pos = Vec::new();
    for ch in from.chars() {
        if oct_string(&from[i..]) {
            pos.push(i);
        }
        i += ch.len_utf8();
    }
    let mut prev = 0;
    let mut ans = String::new();
    for p in pos {
        ans.push_str(&from[prev..p]);
        if let Ok(n) = u32::from_str_radix(&from[p+1..p+4], 8) {
            ans.push(char::from_u32(n).unwrap());
        }
        prev = p + 4;
    }
    ans.push_str(&from[prev..]);
    ans
}

pub fn parse_visible_prompt(prompt: &str) -> (String, String) {
    let mut display = String::new();
    let mut hidden = String::new();
    let mut chars = prompt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'[') {
            chars.next(); // skip '['
            let mut block = String::new();
            while let Some(ch) = chars.next() {
                if ch == '\\' && chars.peek() == Some(&']') {
                    chars.next(); // skip ']'
                    break;
                }
                block.push(ch);
            }
            hidden.push_str(&block);
            if !block.starts_with("\u{1b}]") {
                display.push_str(&block);
            }
        } else {
            display.push(c);
        }
    }
    (display, hidden)
}

pub fn make_prompt_string(raw: &str) -> String {
    let uid = getuid();
    let user = User::from_uid(uid).ok().flatten().map(|u| u.name).unwrap_or_default();
    let hostname = gethostname().ok().and_then(|h| h.to_str().map(|s| s.to_string())).unwrap_or_default();
    let homedir = User::from_uid(uid).ok().flatten().map(|u| file::buf_to_name(&u.dir)).unwrap_or_default();
    let cwd_raw = getcwd().ok().map(|p| file::buf_to_name(&p)).unwrap_or_default();
    let mut cwd = cwd_raw.clone();
    // shorten homedir to '~'
    if cwd.starts_with(&homedir) {
        cwd = cwd.replacen(&homedir, "~", 1);
    }
    let branch = get_branch(&cwd_raw);
    raw.replace("\\u", &user)
       .replace("\\h", &hostname)
       .replace("\\w", &cwd)
       .replace("\\b", &branch)
}

fn get_branch(cwd: &str) -> String {
    let mut path = Path::new(cwd).to_path_buf();
    loop {
        let head = path.join(".git").join("HEAD");
        if let Some(head_str) = head.to_str() {
            if file_check::is_regular_file(head_str) {
                if let Ok(f) = File::open(&head) {
                    let mut line = String::new();
                    let mut reader = BufReader::new(f);
                    if reader.read_line(&mut line).is_ok() {
                        return line.trim().replace("ref: refs/heads/", "") + "🌵";
                    }
                }
            }
        }
        if !path.pop() {
            break;
        }
    }
    String::new()
}