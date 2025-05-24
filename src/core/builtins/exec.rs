//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::unistd;
use nix::errno::Errno;
use std::ffi::CString;

pub fn exec(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if core.db.flags.contains('r') {
        return super::error_exit(1, &args[0], "restricted", core);
    }

    if args.len() == 1 {
        return 0;
    }
    exec_command(&args[1..].to_vec(), core, &"".to_string())
}

fn exec_command(args: &Vec<String>, _: &mut ShellCore, fullpath: &String) -> i32 {
    let cargs: Vec<CString> = args.iter().map(|a| CString::new(a.to_string()).unwrap()).collect();
    let cfullpath = CString::new(fullpath.to_string()).unwrap();

    if ! fullpath.is_empty() {
        let _ = unistd::execv(&cfullpath, &cargs);
    
    }
    let result = unistd::execvp(&cargs[0], &cargs);

    match result {
        Err(Errno::E2BIG) => return 126,
        Err(Errno::EACCES) => return 126,
        Err(Errno::ENOENT) => return 127,
        Err(_) => {
            //eprintln!("Failed to execute. {:?}", err);
            return 127;
        },
        _ => return 127,
    }
}
