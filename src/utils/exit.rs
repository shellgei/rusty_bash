//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::error;
use std::process;

pub fn print(s: &str, core: &mut ShellCore, show_sush: bool) {
    let name = core.data.get_param("0");
    match (core.read_stdin, show_sush) {
        (true, _) => {
            let lineno = core.data.get_param("LINENO");
            eprintln!("{}: line {}: {}", &name, &lineno, s)
        },
        (false, true)  => eprintln!("{}: {}", &name, &s),
        (false, false) => eprintln!("{}", &s),
    }
}


pub fn arg_list_too_long(command_name: &str, core: &mut ShellCore) -> ! {
    let msg = format!("{}: Arg list too long", command_name);
    print(&msg, core, true);
    process::exit(126)
}

pub fn normal(core: &mut ShellCore) -> ! {
    core.write_history_to_file();

    let es_str = core.data.get_param("?");
    let exit_status = match es_str.parse::<i32>() {
        Ok(n)  => n%256,
        Err(_) => {
            let msg = format!("exit: {}: numeric argument required", es_str);
            error::print(&msg, core, true);
            2
        },
    };

    process::exit(exit_status)
}
