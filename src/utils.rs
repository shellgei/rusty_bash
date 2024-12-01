//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod error;
pub mod exit;
pub mod file;
pub mod file_check;
pub mod glob;
pub mod directory;
pub mod random;
pub mod clock;

pub fn reserved(w: &str) -> bool {
    match w {
        "[[" | "]]" | "{" | "}" | "while" | "for" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi" | "case" => true,
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

pub fn consume_with_next_arg(prev_opt: &str, args: &mut Vec<String>) -> String {
    match args.iter().position(|a| a == prev_opt) {
        Some(pos) => {
            match pos+1 >= args.len() {
                true  => {
                    args.remove(pos);
                    "".to_string()
                },
                false => {
                    args.remove(pos);
                    args.remove(pos)
                },
            }
        },
        None => "".to_string(),
    }
}
