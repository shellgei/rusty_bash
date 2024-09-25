//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
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
