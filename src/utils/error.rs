//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::sys::signal::Signal;
use nix::unistd::Pid;

pub enum ParseError {
    UnexpectedSymbol(String),
    UnexpectedEof,
}

pub fn print(s: &str, core: &mut ShellCore) {
    let name = core.db.get_param("0").unwrap();
    if core.db.flags.contains('i') {
        eprintln!("{}: {}", &name, &s);
    }else{
        let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
        eprintln!("{}: line {}: {}", &name, &lineno, s);
    }
}

pub fn internal(s: &str) -> String {
    format!("SUSH INTERNAL ERROR: {}", s)
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

pub fn syntax_in_cond_expr(token: &str) -> String {
    format!("syntax error in conditional expression: unexpected token `{}'", token)
}

pub fn readonly(token: &str) -> String {
    format!("{0}: readonly variable", token)
}

/*
pub fn bad_array_subscript(token: &str) -> String {
    format!("{0}: bad_array_subscript", token)
}
*/

/* error at wait */
pub fn signaled(pid: Pid, signal: Signal, coredump: bool) -> i32 {
    match coredump {
        true  => eprintln!("Pid: {:?}, Signal: {:?} (core dumped)", pid, signal),
        false => eprintln!("Pid: {:?}, Signal: {:?}", pid, signal),
    }
    128+signal as i32
}
