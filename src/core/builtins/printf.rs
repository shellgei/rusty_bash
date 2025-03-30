//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error;
use crate::error::exec::ExecError;
use std::io::{stdout, Write};

#[derive(Debug, Clone)]
enum PrintfToken {
    B(String),
    D(String),
    F(String),
    S(String),
    X(String),
    LargeX(String),
    Q,
    Other(String),
    Normal(String),
    EscapedChar(char),
}

impl PrintfToken {
    fn continue_(&self) -> bool {
        match self {
            Self::Normal(_) | Self::EscapedChar(_) => false,
            _ => true,
        }
    }

    fn to_int(s: &String) -> Result<isize, ExecError> {
        match s.parse::<isize>() {
            Ok(n) => Ok(n),
            Err(_) => Err(ExecError::InvalidNumber(s.to_string())),
        }
    }

    fn to_float(s: &String) -> Result<f64, ExecError> {
        match s.parse::<f64>() {
            Ok(n) => Ok(n),
            Err(_) => Err(ExecError::InvalidNumber(s.to_string())),
        }
    }

    //TODO: implement!
    fn padding_float(_: &mut String, fmt: &mut String) {
        if fmt.is_empty() {
            return;
        }
    }

    fn padding(s: &mut String, fmt: &mut String, is_int: bool) {
        if fmt.is_empty() {
            return;
        }

        let mut right = false;
        let mut padding = ' ';

        if fmt.starts_with("-") {
            fmt.remove(0);
            right = true;
        }
        if fmt.starts_with("0") {
            padding = fmt.remove(0);
        }

        if ! is_int {
            padding = ' ';
        }

        let len = fmt.parse::<usize>().unwrap_or(0);
        if right {
            while s.len() < len {
                s.push(' ');
            }
            return;
        }

        if is_int && s.starts_with("-") && padding == '0' {
            while s.len() < len {
                s.insert(1, '0');
            }
            return;
        }

        while s.len() < len {
            s.insert(0, padding);
        }
    }

    fn to_string(&mut self, args: &mut Vec<String>) -> Result<String, ExecError> {
        match self {
            Self::D(fmt) => {
                let mut a = pop(args);
                Self::padding(&mut a, &mut fmt.clone(), true);
                Ok(a)
            },
            Self::F(fmt) => {
                let mut a = format!("{:.6}", Self::to_float(&pop(args))?);
                Self::padding_float(&mut a, &mut fmt.clone());
                Ok(a)
            },
            Self::S(fmt) => {
                let mut a = pop(args);
                Self::padding(&mut a, &mut fmt.clone(), false);
                Ok(a)
            },
            Self::B(fmt) => {
                let mut a = replace_escape(&pop(args));
                Self::padding(&mut a, &mut fmt.clone(), false);
                Ok(a)
            },
            Self::X(fmt) => {
                let mut a = format!("{:x}", Self::to_int(&pop(args))?);
                Self::padding(&mut a, &mut fmt.clone(), true);
                Ok(a)
            },
            Self::LargeX(fmt) => {
                let mut a = format!("{:X}", Self::to_int(&pop(args))?);
                Self::padding(&mut a, &mut fmt.clone(), true);
                Ok(a)
            },
            Self::Q => {
                let a = pop(args);
                let q = a.replace("\\", "\\\\").replace("$", "\\$").replace("|", "\\|")
                    .replace("\"", "\\\"").replace("'", "\\\'")
                    .replace("(", "\\(").replace(")", "\\)")
                    .replace("{", "\\{").replace("}", "\\}")
                    .replace("!", "\\!").replace("&", "\\&");
                Ok(q)
            },
            Self::Other(s) => {
                let a = pop(args);
                let formatted = match sprintf::sprintf!(&s, a) {
                    Ok(res) => res,
                    Err(e) => {
                        let msg = format!("{} {} {}", &e, &s, &a);
                        return Err(ExecError::Other(msg));
                    },
                };

                Ok(formatted)
            },
            Self::EscapedChar(c) => Ok(esc_to_str(*c)),
            Self::Normal(s) => Ok(s.clone()),
        }
    }
}

fn pop(args: &mut Vec<String>) -> String {
    match args.is_empty() {
        true  => "".to_string(),
        false => args.remove(0),
    }
}

fn esc_to_str(ch: char) -> String {
    match ch {
        'a' => r"\a".to_string(),
        'b' => r"\b".to_string(),
        'e' => r"\e".to_string(),
        'E' => r"\E".to_string(),
        'f' => r"\f".to_string(),
        'n' => "\n".to_string(),
        'r' => "\r".to_string(),
        'v' => r"\v".to_string(),
        't' => "\t".to_string(),
        '\\' => "\\".to_string(),
        _    => ch.to_string(),
    }
}

fn replace_escape(s: &str) -> String {
    let mut ans = String::new();
    let mut esc = false;

    for ch in s.chars() {
        if esc || ch == '\\' {
            if esc {
                ans.push_str(&esc_to_str(ch));
            }
            esc = ! esc;
            continue;
        }

        ans.push(ch);
    }

    ans
}

fn scanner_normal(remaining: &str) -> usize {
    let mut pos = 0;

    for c in remaining.chars() {
        if c == '%' || c == '\\' {
            break;
        }
        pos += c.len_utf8();
    }
    pos
}

fn scanner_escaped_char(remaining: &str) -> usize {
    if ! remaining.starts_with("\\") {
        return 0;
    }

    match remaining.chars().nth(1) {
        Some(ch) => 1 + ch.len_utf8(),
        _ => 0,
    }
}

fn scanner_format_num(remaining: &str) -> usize {
    let mut ans = 0;
    for c in remaining.chars() {
        if ! "-.".contains(c) && (c < '0' || c > '9') {
            break;
        }

        ans += 1;
    }
    ans
}

fn parse(pattern: &str) -> Vec<PrintfToken> {
    let mut remaining = pattern.to_string();
    let mut ans = vec![];

    while ! remaining.is_empty() {
        let len = scanner_normal(&remaining);
        if len > 0 {
            let tail = remaining.split_off(len);
            ans.push(PrintfToken::Normal(remaining));
            remaining = tail;
            continue;
        }

        let len = scanner_escaped_char(&remaining);
        if len > 0 {
            remaining.remove(0);
            ans.push(PrintfToken::EscapedChar(remaining.remove(0)));
            continue;
        }

        if remaining.starts_with("%") {
            remaining.remove(0); // %
                               
            let mut num_part = String::new();
            let len = scanner_format_num(&remaining);
            if len > 0 {
                let tail = remaining.split_off(len);
                num_part = remaining.clone();
                remaining = tail;
            }

            let token = match remaining.chars().next() {
                Some('b') => PrintfToken::B(num_part),
                Some('d') => PrintfToken::D(num_part),
                Some('f') => PrintfToken::F(num_part),
                Some('s') => PrintfToken::S(num_part),
                Some('x') => PrintfToken::X(num_part),
                Some('X') => PrintfToken::LargeX(num_part),
                Some('q') => PrintfToken::Q,
                Some(c)   => PrintfToken::Other("%".to_owned() + &num_part + &c.to_string()),
                None      => PrintfToken::Normal("%".to_string()),
            };

            remaining.remove(0);
            ans.push(token);
        }
    }

    ans
}

fn format(pattern: &str, args: &mut Vec<String>) -> Result<String, ExecError> {
    let mut ans = String::new();

    let mut tokens = parse(pattern);
    let mut fin = true;

    for tok in tokens.iter_mut() {
        if tok.continue_() {
            fin = false;
        }
        ans += &tok.to_string(args)?;
    }

    if ! args.is_empty() && ! fin {
        if let Ok(s) = format(pattern, args) {
            ans += &s;
        }
    }
    Ok(ans)
}

fn arg_check(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 || args[1] == "--help"
    || args[1] == "-v" && args.len() == 3 {
        let msg = format!("printf: usage: printf [-v var] format [arguments]");
        error::print(&msg, core);
        return 2;
    }

    if args[1] == "-v" && args.len() == 2 {
        let msg = format!("printf: -v: option requires an argument");
        error::print(&msg, core);
        let msg = format!("printf: usage: printf [-v var] format [arguments]");
        error::print(&msg, core);
        return 2;
    }

    0
}

fn printf_v(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args[3] == "--" {
        args.remove(3);
    }

    let s = match format(&args[3], &mut args[4..].to_vec()) {
        Ok(ans) => ans,
        Err(e) => {
            let msg = format!("printf: {:?}", e);
            error::print(&msg, core);
            return 1;
        },
    };

    if args[2].contains("[") {
        let tokens = args[2].split('[').collect::<Vec<&str>>();
        let name = tokens[0].to_string();
        let subscript = tokens[1].split(']').nth(0).unwrap().to_string();

        let result = match subscript.parse::<usize>() {
            Ok(n) => core.db.set_array_elem(&name, &s, n, None),
            _ => core.db.set_assoc_elem(&name, &subscript, &s, None),
        };
        if let Err(e) = result {
            let msg = format!("printf: {:?}", e);
            error::print(&msg, core);
            return 2;
        }
        return 0;
    }
    if ! core.db.set_param(&args[2], &s, None).is_ok() {
        return 2;
    }

    return 0;
}

pub fn printf(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match arg_check(core, args) {
        0 => {},
        n => return n,
    }

    if args[1] == "-v" {
        return printf_v(core, args);
    }

    let s = match format(&args[1], &mut args[2..].to_vec()) {
        Ok(ans) => ans,
        Err(e) => {
            let msg = format!("printf: {:?}", e);
            error::print(&msg, core);
            return 1;
        },
    };
    print!("{}", &s);
    stdout().flush().unwrap();
    0
}
