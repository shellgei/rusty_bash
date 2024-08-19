//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn print(s: &str, core: &mut ShellCore) {
    match core.read_stdin {
        true  => {
            let lineno = core.data.get_param("LINENO");
            let msg = format!("line {}: {}", &lineno, s);
            eprintln!("sush: {}", &msg)
        },
        false => eprintln!("sush: {}", &s),
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
