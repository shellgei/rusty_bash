//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause


use crate::ShellCore;
use std::process;

pub trait Error {
    fn print(s: &str, core: &mut ShellCore, show_name: bool) {
        let name = core.data.get_param("0");
        match (core.read_stdin || core.data.flags.contains('c'), show_name) {
            (true, _) => {
                let lineno = core.data.get_param("LINENO");
                eprintln!("{}: line {}: {}", &name, &lineno, s)
            },
            (false, true)  => eprintln!("{}: {}", &name, &s),
            (false, false) => eprintln!("{}", &s),
        }
    }

    fn end(_: &str, _: &mut ShellCore) -> ! {process::exit(1)}
}

pub struct ArgListTooLong;

impl Error for ArgListTooLong {
    fn end(command_name: &str, core: &mut ShellCore) -> ! {
        let msg = format!("{}: Arg list too long", command_name);
        Self::print(&msg, core, true);
        process::exit(126)
    }
}
