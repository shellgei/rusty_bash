//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{proc_ctrl, ShellCore};
use nix::errno::Errno;
use nix::unistd;
use std::ffi::CString;

pub fn exec(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if core.db.flags.contains('r') {
        return super::error_exit(1, &args[0], "restricted", core);
    }

    if args.len() == 1 {
        return 0;
    }

    if core.db.flags.contains('i') || core.shopts.query("execfail") {
        exec_command(&args[1..].to_vec(), core, &"".to_string())
    } else {
        proc_ctrl::exec_command(&args[1..].to_vec(), core, &"".to_string())
    }
}

fn exec_command(args: &Vec<String>, core: &mut ShellCore, fullpath: &String) -> i32 {
    let cargs: Vec<CString> = args
        .iter()
        .map(|a| CString::new(a.to_string()).unwrap())
        .collect();
    let cfullpath = CString::new(fullpath.to_string()).unwrap();

    if !fullpath.is_empty() {
        let _ = unistd::execv(&cfullpath, &cargs);
    }
    let result = unistd::execvp(&cargs[0], &cargs);

    match result {
        Err(Errno::E2BIG) => return super::error_exit(126, &args[0], "Arg list too long", core),
        Err(Errno::EACCES) => {
            return super::error_exit(126, &args[0], "cannot execute: Permission denied", core)
        }
        Err(Errno::ENOENT) => {
            return super::error_exit(127, &args[0], "No such file or directory", core)
        }
        Err(e) => {
            let msg = format!("{:?}", &e);
            return super::error_exit(127, &args[0], &msg, core);
        }
        _ => return 127,
    }
}
