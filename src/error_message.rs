//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

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

pub fn internal_str(s: &str) -> String {
    format!("SUSH INTERNAL ERROR: {}", s)
}

pub fn internal(s: &str) -> ! {
    panic!("{}", internal_str(s))
}

pub fn exponent(s: &str) -> String {
    format!("exponent less than 0 (error token is \"{}\")", s)
}

pub fn recursion(token: &str) -> String {
    format!("{0}: expression recursion level exceeded (error token is \"{0}\")", token)
}

pub fn assignment(right: &str) -> String {
    format!("attempted assignment to non-variable (error token is \"{}\")", right)
}

pub fn syntax(token: &str) -> String {
    format!("{0}: syntax error: operand expected (error token is \"{0}\")", token)
}
