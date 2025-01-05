//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

fn split_format(format: &str) -> Vec<String> {
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

    if &format[len_prev..len] != "" {
        ans.push(format[len_prev..len].to_string());
    }
    ans
}

pub fn printf(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 || args[1] == "--help" {
        eprintln!("printf: usage: printf [-v var] format [arguments]");
        return 2;
    }

    if args[1] == "-v" {
        return 0;
    }

    let mut ans = String::new();
    let parts = split_format(&args[1]);
    let num = std::cmp::min(parts.len(), args.len() - 2);

    for i in 0..num {
        ans += &sprintf::sprintf!(&parts[i], args[2+i]).unwrap();
    }

    if parts.len() == num+1 {
        ans += &parts[num];
    }
    print!("{}", &ans);
    0
}
