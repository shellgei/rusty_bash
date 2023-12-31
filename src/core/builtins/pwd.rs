//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

fn show_pwd(core: &mut ShellCore, physical: bool) -> i32 {
    if let Some(mut path) = core.get_current_directory().clone() {
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

pub fn pwd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 { // $ pwd
        return show_pwd(core, false);
    }

    if &args[1][..1] == "-" { // $pwd -L, pwd -P, pwd hogehoge
        match args[1].as_str() {
            "-P" => return show_pwd(core, true), // シンボリックリンク名を解決して表示する
            "-L" => return show_pwd(core, false), // シンボリックリンク名をそのまま表示する（bash default）
            _ => {
                eprintln!("{}", "sush: pwd: invalid option");
                eprintln!("{}", "pwd: usage: pwd [-LP]");
                return 1;
            },
        }
    }
    show_pwd(core, false)
}
