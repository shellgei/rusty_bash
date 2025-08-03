//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::arg;
use crate::utils::file;
use crate::{error, ShellCore};

pub fn cd(core: &mut ShellCore, args: &[String]) -> i32 {
    if core.db.flags.contains('r') {
        return super::error_exit(1, &args[0], "restricted", core);
    }

    let mut filtered = args.to_vec();
    let _ = arg::consume_option("--", &mut filtered);

    if filtered.len() > 2 {
        eprintln!("sush: cd: too many arguments");
        return 1;
    }

    if filtered.len() == 1 {
        //only "cd"
        return cd_1arg(core, &filtered);
    }

    if filtered[1] == "-" {
        // cd -
        cd_oldpwd(core, &filtered)
    } else {
        // cd /some/dir
        set_oldpwd(core);
        change_directory(core, &filtered)
    }
}

fn cd_1arg(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut extended = args.to_vec();
    extended.push("~".to_string());
    set_oldpwd(core);
    change_directory(core, &extended)
}

fn cd_oldpwd(core: &mut ShellCore, args: &[String]) -> i32 {
    let old = match core.db.get_param("OLDPWD") {
        Ok(old) => {
            println!("{}", &old);
            old.to_string()
        }
        Err(_) => {
            error::print("cd: OLDPWD not set", core);
            return 1;
        }
    };

    let mut modified = args.to_vec();
    modified[1] = old;
    set_oldpwd(core);
    change_directory(core, &modified)
}

fn set_oldpwd(core: &mut ShellCore) {
    if let Some(old) = core.get_current_directory() {
        let _ = core
            .db
            .set_param("OLDPWD", &old.display().to_string(), Some(0));
    };
}

fn change_directory(core: &mut ShellCore, args: &[String]) -> i32 {
    let path = file::make_canonical_path(core, &args[1]);
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
