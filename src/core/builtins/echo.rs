//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::codec::c_string;
use std::io::{stdout, Write};
use std::io;

pub fn echo(_: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut first = true;
    let mut _e_opt = false;
    let mut n_opt = false;

    if args.len() == 1 {
        println!("");
        return 0;
    }

    match args[1].as_ref() {
        "-ne" | "-en" => { _e_opt = true ; n_opt = true ; args.remove(1); },
        "-e" => { _e_opt = true ; args.remove(1); },
        "-n" => { n_opt = true ; args.remove(1); },
        _ => {},
    }

    for a in &args[1..] {
        if ! first {
            let _ = io::stdout().write(b" ");
        }
        first = false;

        let b = c_string::to_carg(a).into_bytes();
        let _ = io::stdout().write_all(&b).unwrap();
    }

    if ! n_opt {
        let _ = io::stdout().write(b"\n");
    }

    stdout().flush().unwrap();
    0
}
