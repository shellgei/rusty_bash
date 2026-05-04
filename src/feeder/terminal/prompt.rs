//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::file;
use crate::{ShellCore, file_check};
use nix::unistd;
use nix::unistd::User;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn oct_string(s: &str) -> bool {
    if !s.starts_with('\\') {
        return false;
    }

    for i in 1..4 {
        match s.chars().nth(i) {
            Some(c) if c.is_ascii_digit() => {}
            _ => return false,
        }
    }
    true
}

fn oct_to_hex_in_str(from: &str) -> String {
    let mut i = 0;
    let mut pos = vec![];

    for ch in from.chars() {
        if oct_string(&from[i..]) {
            pos.push(i);
        }
        i += ch.len_utf8();
    }

    let mut prev = 0;
    let mut ans = String::new();
    for p in pos {
        ans += &from[prev..p];
        if let Ok(n) = u32::from_str_radix(&from[p + 1..p + 4], 8)
            && let Some(ch) = char::from_u32(n)
        {
            ans.push(ch);
        }
        prev = p + 4;
    }
    ans += &from[prev..];
    ans
}

fn git_branch(cwd: &str) -> String {
    let mut dirs: Vec<String> = cwd.split('/').map(str::to_string).collect();
    while !dirs.is_empty() {
        let path = dirs.join("/") + "/.git/HEAD";
        dirs.pop();

        if !file_check::is_regular_file(&path) {
            continue;
        }

        if let Ok(mut f) = File::open(Path::new(&path)) {
            let mut head = String::new();
            if f.read_to_string(&mut head).is_ok() {
                return head.trim_end().replace("ref: refs/heads/", "") + "🌵";
            }
        }
    }

    String::new()
}

pub(super) fn make_prompt_string(core: &mut ShellCore, ps: &str) -> String {
    let raw = core.db.get_param(ps).unwrap_or_default();
    let raw = oct_to_hex_in_str(&raw);

    let uid = unistd::getuid();
    let user_info = User::from_uid(uid).ok().flatten();
    let user = user_info.as_ref().map(|u| u.name.as_str()).unwrap_or("");
    let hostname = match unistd::gethostname() {
        Ok(h) => file::oss_to_name(&h),
        _ => String::new(),
    };

    let homedir = user_info
        .as_ref()
        .map(|u| file::buf_to_name(&u.dir))
        .unwrap_or_default();
    let mut cwd = match unistd::getcwd() {
        Ok(p) => file::buf_to_name(&p),
        _ => String::new(),
    };
    let branch = git_branch(&cwd);

    if cwd.starts_with(&homedir) {
        cwd = cwd.replacen(&homedir, "~", 1);
    }

    raw.replace("\\u", user)
        .replace("\\h", &hostname)
        .replace("\\w", &cwd)
        .replace("\\b", &branch)
}
