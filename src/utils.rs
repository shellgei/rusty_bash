//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod glob;
pub mod directory;

pub fn reserved(w: &str) -> bool {
    match w {
        "{" | "}" | "while" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi" | "case" => true,
        _ => false,
    }
}

pub fn split_words(s: &str) -> Vec<String> {
    let mut ans = vec![];

    let mut in_quote = false;
    let mut escaped = false;
    let mut quote = ' ';

    let mut tmp = String::new();
    for c in s.chars() {
        if escaped || c == '\\' {
            escaped = ! escaped;
            tmp.push(c);
            continue;
        }

        if c == '\'' || c == '"' {
            if c == quote {
                in_quote = ! in_quote;
                quote = ' ';
            }else if quote == ' ' {
                in_quote = ! in_quote;
                quote = c;
            }
            tmp.push(c);
            continue;
        }

        if in_quote {
            tmp.push(c);
            continue;
        }

        if ! in_quote && ( c == ' ' || c == '\t') {
            if tmp.len() != 0 {
                ans.push(tmp.clone());
                tmp.clear();
            }
        }else{
            tmp.push(c);
        }
    }

    if tmp.len() != 0 {
        ans.push(tmp);
    }

    ans
}

pub fn is_wsl() -> bool {
    if let Ok(info) = nix::sys::utsname::uname() {
        let release = info.release().to_string_lossy().to_string();
        return release.find("WSL").is_some();
    };

    false
}

/*
pub fn str_to_i64(s: &str) -> Option<i64> {
    if s.len() == 0 {
        return Some(0);
    }

    let mut body = s.to_string(); 
    let mut sign = match body.starts_with("-") || body.starts_with("+") {
        true  => body.remove(0).to_string(),
        false => "".to_string(),
    };

    let mut base = 10;

    if s.starts_with("0x") || s.starts_with("0X") {
        base = 16;
        body.remove(0);
        body.remove(0);
    }else if s.starts_with("0") {
        base = 8;
        body.remove(0);
    }else if s.chars().all(|ch| ch >= '0' && ch <= '9') {
        let splits = s.split("#");
    }


    if ! s.chars().all(|ch| ch >= '0' && ch <= '9') {
        return None;
    }

    Some(0)
}
*/
