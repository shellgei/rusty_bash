//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error;
use sprintf::PrintfError;
use std::io::{stdout, Write};

fn split_format(format: &str) -> (Vec<String>, Option<String>) {
    let mut escaped = false;
    let mut percent = false;
    let mut len = 0;
    let mut len_prev = 0;
    let mut ans = vec![];

    for c in format.chars() {
        if c == '\\' || escaped {
            len += c.len_utf8();
            escaped = ! escaped;
            percent = false;
            continue;
        }

        len += c.len_utf8();
        if c == '%' {
            percent = true;
            continue;
        }

        if percent {
            ans.push(format[len_prev..len].to_string());
            len_prev = len;
            percent = false;
        }
    }

    match format[len_prev..len].is_empty() {
        true  => (ans, None),
        false => (ans, Some(format[len_prev..len].to_string()) ),
    }
}

fn output(pattern: &str, args: &mut Vec<String>) -> Result<String, PrintfError> {
    let mut ans = String::new();
    let (parts, tail) = split_format(&pattern);

    while parts.len() > args.len() {
        args.push(String::new());
    }

    for i in 0..parts.len() {
        if parts[i].contains("%d") {
            if let Ok(_) = args[i].parse::<i32>() {
                ans += &parts[i].replace("%d", &args[i]);
            }
        }else {
            ans += &sprintf::sprintf!(&parts[i], args[i])?;
        }
    }

    if let Some(s) = tail {
        ans += &s;
    }
    Ok(ans)
}

pub fn printf(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
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

    if args[1] == "-v" {
        let s = match output(&args[3], &mut args[4..].to_vec()) {
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

    let s = match output(&args[1], &mut args[2..].to_vec()) {
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
