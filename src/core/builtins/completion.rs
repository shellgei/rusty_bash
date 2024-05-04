//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn compgen(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    match args[1].as_str() {
        "-W" => compgen_large_w(core, args),
        _ => {
            eprintln!("sush: compgen: {}: invalid option", &args[1]);
            return 2;
        },
    }
}

pub fn compgen_large_w(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    0
}
