//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod arg;
pub mod clock;
pub mod directory;
pub mod exit;
pub mod file;
pub mod file_check;
pub mod glob;
pub mod restricted_shell;
pub mod string_binary;

use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error::exec::ExecError;
use crate::error::input::InputError;
use crate::{Feeder, Script, ShellCore};
use faccess::PathExt;
use io_streams::StreamReader;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use std::{thread, time};

pub fn reserved(w: &str) -> bool {
    matches!(
        w,
        "[[" | "]]"
            | "{"
            | "}"
            | "while"
            | "until"
            | "for"
            | "do"
            | "done"
            | "if"
            | "then"
            | "elif"
            | "else"
            | "fi"
            | "case"
            | "coproc"
            | "esac"
            | "repeat"
            | "select"
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

    is_var(s)
}

pub fn is_var(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first_ch = s.chars().next().unwrap();
    if first_ch.is_ascii_digit() {
        return false;
    }

    let name_c = |c: char| {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || '_' == c
    };
    !s.chars().any(|c| !name_c(c))
}

pub fn read_line_stdin_unbuffered(
    delim: &str,
    timeout: Option<f32>,
    subshell: bool,
    limit: &mut usize,
) -> Result<String, InputError> {
    if let Some(timeout) = timeout {
        if subshell {
            return read_line_stdin_unbuffered_thread(delim, timeout, limit);
        } else {
            return read_line_stdin_unbuffered_nonblock(delim, timeout, limit);
        }
    }

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
                *limit -= 1;
                if d == ch[0] || *limit == 0 {
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

pub fn read_line_stdin_unbuffered_nonblock(
    delim: &str,
    timeout: f32,
    limit: &mut usize,
) -> Result<String, InputError> {
    let start = clock::monotonic_time();

    let mut stdin = termion::async_stdin();
    let mut line = vec![];
    let mut ch: [u8; 1] = Default::default();

    let mut d = 10; //\n
    if let Some(Ok(c)) = delim.as_bytes().bytes().next() {
        d = c;
    }

    loop {
        let cur = clock::monotonic_time();

        if (cur - start).as_secs_f32() > timeout {
            return Err(InputError::Timeout);
        }

        if timeout > 0.001 {
            thread::sleep(time::Duration::from_millis(1));
        }

        match stdin.read(&mut ch) {
            Ok(0) => {}
            Ok(_) => {
                line.push(ch[0]);
                *limit -= 1;
                if d == ch[0] || *limit == 0 {
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

pub fn read_line_stdin_unbuffered_thread(
    delim: &str,
    timeout: f32,
    limit: &mut usize,
) -> Result<String, InputError> {
    let (tx, rx) = mpsc::channel();
    let delim = delim.to_string();

    let mut limit_internal = *limit;

    thread::spawn(move || {
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
                    limit_internal -= 1;
                    if d == ch[0] || limit_internal == 0 {
                        break;
                    }
                }
                Err(_) => return Err(InputError::Eof),
            }
        }

        match String::from_utf8(line) {
            Ok(s) => {
                let _: () = tx.send(s).unwrap();
                Ok(())
            }
            Err(_) => Err(InputError::Eof),
        }
    });

    match rx.recv_timeout(Duration::from_secs_f32(timeout)) {
        Ok(line) => {
            *limit -= line.len();
            Ok(line)
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(InputError::Timeout),
        Err(_) => Err(InputError::Eof),
    }
}

pub fn to_ansi_c(s: &str) -> String {
    let mut ans = String::new();
    let mut ansi = false;
    let mut double_quote = false;

    for c in s.chars() {
        match c as usize {
            bin @ 0..9 => {
                ansi = true;
                let alter = format!("\\{bin:03o}");
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
    } else if double_quote {
        ans.insert(0, '"');
        ans.push('"');
    }

    ans
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

pub fn string_to_calculated_string(from: &str, core: &mut ShellCore) -> Result<String, ExecError> {
    let mut f = Feeder::new(from);
    if let Some(mut a) = ArithmeticExpr::parse(&mut f, core, false, "")?
        && f.is_empty()
    {
        return a.eval(core);
    }

    Err(ExecError::SyntaxError(f.consume(f.len())))
}

pub fn gen_not_exist_var(core: &mut ShellCore) -> String {
    let mut nm = "fjoeeojwa".to_string();
    while core.db.exist(&nm) || core.db.exist_nameref(&nm) {
        nm.push('a');
    }
    nm
}

pub fn groups() -> Vec<String> {
    let num = unsafe { libc::getgroups(0, ::std::ptr::null_mut()) };
    let mut groups = vec![0; num as usize];
    unsafe { libc::getgroups(num, groups.as_mut_ptr()) };
    groups.iter().map(|e| e.to_string()).collect()
}

pub fn run_error_script(core: &mut ShellCore) {
    if core.error_script.is_empty() {
        return;
    }

    core.error_script_run = true;
    let mut feeder = Feeder::new(&core.error_script);
    match Script::parse(&mut feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        }
        Err(e) => {
            e.print(core);
        }
        Ok(None) => {}
    };

    core.db.exit_status = 0;
    core.error_script_run = false;
}

pub fn run_debug_script(core: &mut ShellCore) {
    if core.debug_script.is_empty() 
    || core.debug_script_run
    || core.is_subshell {
        return;
    }

    core.debug_script_run = true;
    let mut feeder = Feeder::new(&core.debug_script);
    let lineno = core.db.get_param("LINENO").unwrap_or("0".to_string()).parse::<usize>().unwrap_or(0);
    feeder.lineno += lineno - 1;
    let bkup = core.debug_script.clone();
    core.debug_script.clear();
    match Script::parse(&mut feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        }
        Err(e) => {
            e.print(core);
        }
        Ok(None) => {}
    };

    //core.db.exit_status = 0;
    core.debug_script = bkup;
    core.debug_script_run = false;
}
