//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn shopt(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
        core.shopts.print_all();
        return 0;
    }

    if args[1] == "-s" {
        if args.len() == 2 {
            core.shopts.print_if(true);
        }else{
            core.shopts.set(&args[2], true);
        }
        return 0;
    }

    if args[1] == "-u" {
        if args.len() == 2 {
            core.shopts.print_if(false);
        }else{
            core.shopts.set(&args[2], false);
        }
        return 0;
    }

    0
}
