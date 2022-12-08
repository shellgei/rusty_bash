//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::process;
use crate::ShellCore;

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.vars.insert("?".to_string(), args[1].clone());
    }

    let exit_status = match core.vars["?"].parse::<i32>() {
        Ok(n)  => n%256, 
        Err(_) => {
            eprintln!("sush: exit: {}: numeric argument required", core.vars["?"]);
            2
        },
    };

    process::exit(exit_status);
}
