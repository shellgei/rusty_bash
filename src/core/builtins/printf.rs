//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

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

fn output(pattern: &str, args: &mut Vec<String>) -> String {
    let mut ans = String::new();
    let (parts, tail) = split_format(&pattern);

    while parts.len() > args.len() {
        args.push(String::new());
    }

    for i in 0..parts.len() {
        ans += &sprintf::sprintf!(&parts[i], args[i]).unwrap();
    }

    if let Some(s) = tail {
        ans += &s;
    }
    ans
}

pub fn printf(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 || args[1] == "--help" {
        eprintln!("printf: usage: printf [-v var] format [arguments]");
        return 2;
    }

    if args[1] == "-v" {
        let s = output(&args[3], &mut args[4..].to_vec());
        if ! core.db.set_param(&args[2], &s).is_ok() {
            return 2;
        }

        return 0;
    }

    let s = output(&args[1], &mut args[2..].to_vec());
    print!("{}", &s);
    0
}
