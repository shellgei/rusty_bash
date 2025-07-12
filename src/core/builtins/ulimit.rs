//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, ShellCore};

fn print(_: &mut ShellCore) -> i32 {
    0
}

pub fn ulimit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let _ = arg::dissolve_options(args);

    if args.iter().any(|a| a == "-a"){
        return print(core);
    }

    0
}

