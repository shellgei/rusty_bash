//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod directory;
pub mod clock;
pub mod exit;
pub mod file;
pub mod file_check;
pub mod glob;
pub mod arg;
pub mod restricted_shell;
pub mod splitter;
pub mod c_string;

use crate::{Feeder, ShellCore};
use crate::error::input::InputError;
use crate::error::exec::ExecError;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use faccess::PathExt;
use io_streams::StreamReader;
use std::io::Read;
use std::path::Path;

pub fn reserved(w: &str) -> bool {
    match w {
        "[[" | "]]" | "{" | "}" | "while" | "for" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi" | "case" | "esac" | "repeat" => true,
        _ => false,
    }
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
            end_with_space = true;
            if ! tmp.is_empty() {
                ans.push(tmp.clone());
                tmp.clear();
            }
        }else{
            tmp.push(c);
        }
    }

    if ! tmp.is_empty() {
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
        return release.find("WSL").is_some();
    };

    false
}

pub fn is_name(s: &str, core: &mut ShellCore) -> bool {
    let mut f = Feeder::new(s);
    s.len() > 0 && f.scanner_name(core) == s.len()
}

pub fn is_param(s :&str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_ch = s.chars().nth(0).unwrap();
    if s.len() == 1 { //special or position param
        if "$?*@#-!_0123456789".find(first_ch) != None {
            return true;
        }
    }else {
        if let Ok(n) = s.parse::<usize>() {
            return n > 0;
        }
    }

    /* variable */
    if '0' <= first_ch && first_ch <= '9' {
        return false;
    }

    let name_c = |c| ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z')
                     || ('0' <= c && c <= '9') || '_' == c;
    s.chars().position(|c| !name_c(c)) == None
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
            },
            Ok(_) => {
                line.push(ch[0]);
                if d == ch[0] {
                    break;
                }
            },
            Err(_) => return Err(InputError::Eof),
        }
    }

    match String::from_utf8(line) {
        Ok(s) => Ok(s),
        Err(_) => Err(InputError::NotUtf8),
    }
}

pub fn to_ansi_c(s: &String) -> String {
    let mut ans = String::new();
    let mut ansi = false;
    let mut double_quote = false;

    for c in s.chars() {
        match c as usize {
            bin @ 0..9 => {
                ansi = true;
                let alter = format!("\\{:03o}", bin);
                ans.push_str(&alter);
            },
            9 => {
                ansi = true;
                ans.push_str("\\t");
            },
            0x0A => {
                ansi = true;
                ans.push_str("\\n");
            },
            0x22 | 0x24 | 0x60 => { // "
                double_quote = true;
                ans.push('\\');
                ans.push(c);
            },
            0x20 | /*0x27 |*/ 0x2A | 0x40 | 0x5B | 0x5D => { // space, ' , * , @, [ , ],
                double_quote = true;
                ans.push(c);
            },
            _ => ans.push(c),
        }
    }

    if ansi {
        ans.insert(0, '\'');
        ans.insert(0, '$');
        ans.push('\'');
    }else if double_quote {
        ans.insert(0, '"');
        ans.push('"');
    }

    ans
}

pub fn get_command_path(s: &String, core: &mut ShellCore) -> String {
    for path in core.db.get_param("PATH").unwrap_or(String::new()).to_string().split(":") {
        for command in directory::files(path).iter() {
            let fullpath = path.to_owned() + "/" + command;
            if ! Path::new(&fullpath).executable() {
                continue;
            }

            if command == s {
                return fullpath;
            }
        }
    }

    String::new()
}


pub fn string_to_calculated_string(from: &str, core: &mut ShellCore)
-> Result<String, ExecError> {
    let mut f = Feeder::new(from);
    if let Some(mut a) = ArithmeticExpr::parse(&mut f, core, false, "")? {
        if f.is_empty() {
            return a.eval(core);
        }
    }
 
    Err(ExecError::SyntaxError(f.consume(f.len())))
}
