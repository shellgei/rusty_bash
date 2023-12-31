//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod utils;
mod pwd;

use crate::ShellCore;

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("cd".to_string(), cd);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("true".to_string(), true_);
    }
}

pub fn cd(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 0 {
        eprintln!("SUSH INTERNAL ERROR: (no arg for cd)");
        return 1;
    }
    if args.len() > 2 {
        eprintln!("sush: cd: too many arguments");
        return 1;
    }

    if args.len() == 1 { //only "cd"
        let var = "~".to_string();
        args.push(var);
    }else if args.len() == 2 && args[1] == "-" { // cd -
        if let Some(old) = core.vars.get("OLDPWD") {
            println!("{}", &old);
            args[1] = old.to_string();
        }
    };

    if let Some(old) = core.get_current_directory().clone() {
        core.vars.insert("OLDPWD".to_string(), old.display().to_string());
    };

    let path = utils::make_canonical_path(utils::make_absolute_path(core, &args[1]));
    if core.set_current_directory(&path).is_ok() {
        core.vars.insert("PWD".to_string(), path.display().to_string());
        0
    }else{
        eprintln!("sush: cd: {:?}: No such file or directory", &path);
        1
    }
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.vars.insert("?".to_string(), args[1].clone());
    }
    core.exit()
}

pub fn false_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    1
}

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}
