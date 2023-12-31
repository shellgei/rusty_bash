//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn pwd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut physical: bool = false;

    match args.len() {
        0 => {
            eprintln!("Bug of this shell");
            return 1;    
        },
        2 => {
            if &args[1][..1] == "-" {
                match args[1].as_str() {
                    "-P" => { physical = true }, // シンボリックリンク名を解決して表示する
                    "-L" => (), // シンボリックリンク名をそのまま表示する（bash default）
                    _ => {
                        eprintln!("{}", "sush: pwd: invalid option");
                        eprintln!("{}", "pwd: usage: pwd [-LP]");
                        return 1;
                    },
                }
            }
        },
        _ => (),
    }

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
