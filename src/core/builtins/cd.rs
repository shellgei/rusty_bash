//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::{arg, file};
use crate::{error, ShellCore};

pub fn cd(core: &mut ShellCore, args: &[String]) -> i32 {
    if core.db.flags.contains('r') {
        return super::error_(1, &args[0], "restricted", core);
    }

    let mut args = args.to_owned();
    arg::consume_arg("--", &mut args);

    if args.len() > 2 {
        eprintln!("sush: cd: too many arguments");
        return 1;
    }

    // only "cd"
    if args.len() == 1 {
        set_oldpwd(core);
        let home = core.db.get_param("HOME").unwrap_or_default();
        return change_directory(core, &home);
    }

    // cd -
    if args[1] == "-" {
        return cd_oldpwd(core);
    }

    // cd /some/dir
    set_oldpwd(core);
    change_directory(core, &args[1])
}

fn cd_oldpwd(core: &mut ShellCore) -> i32 {
    match core.db.get_param("OLDPWD") {
        Ok(old) => {
            println!("{}", &old);
            set_oldpwd(core);
            change_directory(core, &old)
        }
        Err(_) => {
            error::print("cd: OLDPWD not set", core);
            1
        }
    }
}

fn set_oldpwd(core: &mut ShellCore) {
    if let Some(old) = core.get_current_directory() {
        let _ = core
            .db
            .set_param("OLDPWD", &old.display().to_string(), Some(0));
    };
}

fn change_directory(core: &mut ShellCore, target: &str) -> i32 {
    let path = file::make_canonical_path(core, target);
    if core.set_current_directory(&path).is_ok() {
        let _ = core
            .db
            .set_param("PWD", &path.display().to_string(), Some(0));
        0
    } else {
        eprintln!("sush: cd: {:?}: No such file or directory", &path);
        1
    }
}
