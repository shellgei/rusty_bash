//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

fn caller_no_arg(core: &mut ShellCore) -> i32 {
    let linenos_len = core.db.index_based_len("BASH_LINENO");
    if linenos_len == 0 {
        return 1;
    }

    let lineno = core.db.get_elem("BASH_LINENO", "0").unwrap();
    let mut funcname = core.db.get_elem("FUNCNAME", "1").unwrap();

    if funcname == "" {
        funcname = "NULL".to_string();
    }else {
        funcname = "main".to_string();
    }

    println!("{} {}", &lineno, funcname);
    0
}

fn caller_arg(core: &mut ShellCore, args: &[String]) -> i32 {
    let pos = match args[1].parse::<usize>() {
        Ok(n) => n,
        _ => return 1,
    };

    let linenos_len = core.db.index_based_len("BASH_LINENO");
    if linenos_len == 0 {
        return 1;
    }
    //let functions_len = core.db.index_based_len("FUNCNAME");


    let lineno = core.db.get_elem("BASH_LINENO", &pos.to_string()).unwrap();
    let funcname = core.db.get_elem("FUNCNAME", &(pos+1).to_string()).unwrap();

    let mut script_name = core.script_name.clone();
    if script_name == "-" {
        script_name = "main".to_string();
    }

    println!("{} {} {}", &lineno, funcname, script_name);
    0
}

pub fn caller(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() == 1 {
        caller_no_arg(core)
    }else{
        caller_arg(core, args)
    }
}
