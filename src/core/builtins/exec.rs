//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{proc_ctrl, ShellCore};
use nix::errno::Errno;
use nix::unistd;
use std::ffi::CString;

pub fn exec(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    if core.db.flags.contains('r') {
        return super::error_exit_text(1, &args[0], "restricted", core);
    }

    if args.len() == 1 {
        return 0;
    }

    if core.db.flags.contains('i') || core.shopts.query("execfail") {
        exec_command(&args[1..], core, "")
    } else {
        proc_ctrl::exec_command(&args[1..], core, "")
    }
}

fn exec_command(args: &[String], core: &mut ShellCore, fullpath: &str) -> i32 {
    let cargs: Vec<CString> = args
        .iter()
        .map(|a| CString::new(a.to_string()).unwrap())
        .collect();
    let cfullpath = CString::new(fullpath).unwrap();

    if !fullpath.is_empty() {
        let _ = unistd::execv(&cfullpath, &cargs);
    }
    let result = unistd::execvp(&cargs[0], &cargs);

    match result {
        Err(Errno::E2BIG) => super::error_exit_text(126, &args[0], "Arg list too long", core),
        Err(Errno::EACCES) => {
            super::error_exit_text(126, &args[0], "cannot execute: Permission denied", core)
        }
        Err(Errno::ENOENT) => super::error_exit_text(127, &args[0], "No such file or directory", core),
        Err(e) => {
            let msg = format!("{:?}", &e);
            super::error_exit_text(127, &args[0], &msg, core)
        }
        _ => 127,
    }
}
