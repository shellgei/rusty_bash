//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::arg;
use crate::ShellCore;

fn print_enabled(core: &mut ShellCore) -> i32 {
    let mut list = core.builtins.keys().map(|e| e.to_string()).collect::<Vec<String>>();
    list.append(&mut core.subst_builtins.keys().map(|e| e.to_string()).collect::<Vec<String>>());
    list.sort();

    list.iter().for_each(|e| println!("enable {e}"));
    0
}

pub fn enable(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    let args = arg::dissolve_options(&args);

    if args.len() < 2 {
        return print_enabled(core);
    }

    0
}
