//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::{arg, error};

fn is_varname(s :&String) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_ch = s.chars().nth(0).unwrap();

    if '0' <= first_ch && first_ch <= '9' {
        return false;
    }

    let name_c = |c| ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z')
                     || ('0' <= c && c <= '9') || '_' == c;
    s.chars().position(|c| !name_c(c)) == None
}

pub fn read(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    let mut args = arg::dissolve_options(args);
    let _r_opt = arg::consume_option("-r", &mut args); //TODO: change the precedure

    for a in &args[1..] {
        if ! is_varname(&a) {
            eprintln!("bash: read: `{}': not a valid identifier", &a);
            return 1;
        }else{
            if let Err(e) = core.db.set_param(&a, "", None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
        }
    }

    //TODO: this procedure may be for -r option
    let mut line = String::new();
    let len = std::io::stdin()
        .read_line(&mut line)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");

    let mut pos = 1;
    let mut overflow = String::new();
    for w in line.trim_end().split(' ') {
        if pos < args.len()-1 {
            if let Err(e) = core.db.set_param(&args[pos], &w, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
            pos += 1;
        }else{
            if overflow.len() != 0 {
                overflow += " ";
            }
            overflow += &w;
            if let Err(e) = core.db.set_param(&args[pos], &overflow, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
        }
    }

    match len == 0 {
        true  => 1,
        false => 0,
    }
}
