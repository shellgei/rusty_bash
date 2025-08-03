//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod arg;
pub mod clock;
pub mod directory;
pub mod exit;
pub mod file;
pub mod file_check;
pub mod glob;
pub mod splitter;

use crate::error::input::InputError;
use crate::{Feeder, ShellCore};
use faccess::PathExt;
use io_streams::StreamReader;
use std::io::Read;
use std::path::Path;

pub fn reserved(w: &str) -> bool {
    matches!(
        w,
        "[[" | "]]"
            | "{"
            | "}"
            | "while"
            | "for"
            | "do"
            | "done"
            | "if"
            | "then"
            | "elif"
            | "else"
            | "fi"
            | "case"
            | "esac"
            | "repeat"
    )
}

pub fn split_words(s: &str) -> Vec<String> {
    let mut ans = vec![];
    let mut end_with_space = false;

    let mut in_quote = false;
    let mut escaped = false;
    let mut quote = ' ';

    let mut tmp = String::new();
    for c in s.chars() {
        end_with_space = false;
        if escaped || c == '\\' {
            escaped = !escaped;
            tmp.push(c);
            continue;
        }

        if c == '\'' || c == '"' {
            if c == quote {
                in_quote = !in_quote;
                quote = ' ';
            } else if quote == ' ' {
                in_quote = !in_quote;
                quote = c;
            }
            tmp.push(c);
            continue;
        }

        if in_quote {
            tmp.push(c);
            continue;
        }

        if !in_quote && (c == ' ' || c == '\t') {
            end_with_space = true;
            if !tmp.is_empty() {
                ans.push(tmp.clone());
                tmp.clear();
            }
        } else {
            tmp.push(c);
        }
    }

    if !tmp.is_empty() {
        ans.push(tmp);
    }

    if end_with_space {
        ans.push("".to_string());
    }

    ans
}

pub fn is_wsl() -> bool {
    if let Ok(info) = nix::sys::utsname::uname() {
        let release = info.release().to_string_lossy().to_string();
        return release.contains("WSL");
    };

    false
}

pub fn is_name(s: &str, core: &mut ShellCore) -> bool {
    let mut f = Feeder::new(s);
    !s.is_empty() && f.scanner_name(core) == s.len()
}

pub fn is_param(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_ch = s.chars().next().unwrap();
    if s.len() == 1 {
        //special or position param
        if "$?*@#-!_0123456789".find(first_ch).is_some() {
            return true;
        }
    } else if let Ok(n) = s.parse::<usize>() {
        return n > 0;
    }

    /* variable */
    if first_ch.is_ascii_digit() {
        return false;
    }

    let name_c = |c: char| {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || '_' == c
    };
    !s.chars().any(|c| !name_c(c))
}

pub fn read_line_stdin_unbuffered(delim: &str) -> Result<String, InputError> {
    let mut line = vec![];
    let mut ch: [u8; 1] = Default::default();
    let mut stdin = StreamReader::stdin().unwrap();

    let mut d = 10; //\n
    if let Some(Ok(c)) = delim.as_bytes().bytes().next() {
        d = c;
    }

    loop {
        match stdin.read(&mut ch) {
            Ok(0) => {
                if line.is_empty() {
                    return Err(InputError::Eof);
                }
                break;
            }
            Ok(_) => {
                line.push(ch[0]);
                if d == ch[0] {
                    break;
                }
            }
            Err(_) => return Err(InputError::Eof),
        }
    }

    match String::from_utf8(line) {
        Ok(s) => Ok(s),
        Err(_) => Err(InputError::NotUtf8),
    }
}

pub fn to_ansi_c(s: &str) -> String {
    if s.contains('\n') {
        //TODO: add \t \a ...
        return "$'".to_owned() + &s.replace("\n", "\\n") + "'";
    }
    s.to_string()
}

pub fn get_command_path(s: &str, core: &mut ShellCore) -> String {
    for path in core.db.get_param("PATH").unwrap_or_default().split(":") {
        for command in directory::files(path).iter() {
            let fullpath = path.to_owned() + "/" + command;
            if !Path::new(&fullpath).executable() {
                continue;
            }

            if command == s {
                return fullpath;
            }
        }
    }

    String::new()
}
