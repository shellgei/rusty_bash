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

fn disable(core: &mut ShellCore, commands: &[String]) -> i32 {
    for com in commands {
        if let Some(func) = core.builtins.remove(com) {
            core.disabled_builtins.insert(com.to_string(), func);
        }else if let Some(func) = core.subst_builtins.remove(com) {
            core.disabled_subst_builtins.insert(com.to_string(), func);
        }
    }
    0
}

pub fn enable(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    let args = arg::dissolve_options(&args);

    if args.len() < 2 {
        return print_enabled(core);
    }else if args[1] == "-n" {
        disable(core, &args[2..]);
    }

    0
}
