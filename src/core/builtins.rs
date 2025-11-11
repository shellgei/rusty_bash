//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod cd;
mod variable;
mod pwd;
mod utils;

use crate::ShellCore;
use crate::utils::exit;

pub fn error_exit(exit_status: i32, name: &str,
                  msg: &str, core: &mut ShellCore) -> i32 {
    let shellname = core.db.get_param("0").unwrap();
    eprintln!("{}: {}: {}", &shellname, name, msg);
    exit_status
}

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("return".to_string(), return_);
        self.builtins.insert("true".to_string(), true_);

        self.subst_builtins
            .insert("local".to_string(), variable::local);
    }
}

pub fn exit(core: &mut ShellCore, args: &[String]) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        let _ = core.db.set_param("?", &args[1], None);
    }
    exit::normal(core)
}

pub fn false_(_: &mut ShellCore, _: &[String]) -> i32 {
    1
}

pub fn return_(core: &mut ShellCore, args: &[String]) -> i32 {
    if core.source_function_level <= 0 {
        eprintln!("sush: return: can only `return' from a function or sourced script");
        return 2;
    }
    core.return_flag = true;

    if args.len() < 2 {
        return 0;
    }else if let Ok(n) = args[1].parse::<i32>() {
        return n%256;
    }

    eprintln!("sush: return: {}: numeric argument required", args[1]);
    2
}

pub fn true_(_: &mut ShellCore, _: &[String]) -> i32 {
    0
}
