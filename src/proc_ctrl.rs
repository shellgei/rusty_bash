//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::c_string;
use nix::errno::Errno;
use nix::unistd;
use std::process;

pub fn exec_command(args: &[String]) -> ! {
    let cargs = c_string::to_cargs(args);
    let result = unistd::execvp(&cargs[0], &cargs);

    match result {
        Err(Errno::EACCES) => {
            println!("sush: {}: Permission denied", &args[0]);
            process::exit(126)
        },
        Err(Errno::ENOENT) => {
            println!("{}: command not found", &args[0]);
            process::exit(127)
        },
        Err(err) => {
            println!("Failed to execute. {:?}", err);
            process::exit(127)
        }
        _ => panic!("SUSH INTERNAL ERROR (never come here)")
    }
}
