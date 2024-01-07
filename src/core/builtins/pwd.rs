//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn pwd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 || &args[1][..1] != "-" { // $ pwd, $ pwd aaa
        return show_pwd(core, false);
    }

    match args[1].as_str() { //$pwd -L, pwd -P, pwd -aaaa
        "-P" => show_pwd(core, true), // シンボリックリンク名を解決して表示
        "-L" => show_pwd(core, false), // シンボリックリンク名をそのまま表示（bash default）
        _ => {
            eprintln!("sush: pwd: {}: invalid option", &args[1]);
            eprintln!("pwd: usage: pwd [-LP]");
            1
        },
    }
}

fn show_pwd(core: &mut ShellCore, physical: bool) -> i32 {
    if let Some(mut path) = core.get_current_directory() {
        if physical && path.is_symlink() {
            if let Ok(c) = path.canonicalize() {
                path = c;
            }
        }
        println!("{}", path.display());
        return 0;
    }
    1
}
