//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::utils;

pub fn cd(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() > 2 {
        eprintln!("sush: cd: too many arguments");
        return 1;
    }

    if args.len() == 1 { //only "cd"
        return cd_1arg(core, args);
    }

    if args[1] == "-" { // cd -
        cd_oldpwd(core, args)
    }else{ // cd /some/dir
        set_oldpwd(core);
        change_directory(core, args)
    }
}

fn cd_1arg(core: &mut ShellCore, args: &[String]) -> i32 {
    let var = "~".to_string();
    let mut args = args.to_vec();
    args.push(var);
    set_oldpwd(core);
    change_directory(core, &args)
}

fn cd_oldpwd(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = args.to_vec();
    if let Ok(old) = core.db.get_param("OLDPWD") {
        println!("{}", &old);
        args[1] = old.to_string();
    }else {
        eprintln!("sush: cd: OLDPWD not set");
        return 1;
    }

    set_oldpwd(core);
    change_directory(core, &args)
}

fn set_oldpwd(core: &mut ShellCore) {
    if let Some(old) = core.get_current_directory() {
        let _ = core.db.set_param("OLDPWD", &old.display().to_string(), None);
    };
}

fn change_directory(core: &mut ShellCore, args: &[String]) -> i32 {
    let path = utils::make_canonical_path(core, &args[1]);
    if core.set_current_directory(&path).is_ok() {
        let _ = core.db.set_param("PWD", &path.display().to_string(), None);
        0
    }else{
        let msg = match core.db.get_param("LANG").as_deref() {
            Ok("ja_JP.UTF-8") => "そのようなファイルやディレクトリはありません",
            _  => "No such file or directory",
        };

        eprintln!("sush: cd: {:?}: {}", &path, msg);
        1
    }
}
