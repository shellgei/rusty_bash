//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use nix::sys::signal::Signal;
use nix::unistd::Pid;

#[derive(Debug)]
pub enum ExecError {
    Internal,
    ArrayIndexInvalid(String),
    BadSubstitution(String),
    DivZero,
    Exponent(i64),
    InvalidBase(String),
    InvalidName(String),
    VariableReadOnly(String),
    VariableInvalid(String),
    OperandExpected(String),
    SubstringMinus(i64),
    Other(String),
}

impl From<ExecError> for String {
    fn from(e: ExecError) -> String {
        match e {
            ExecError::Internal => "INTERNAL ERROR".to_string(),
            ExecError::ArrayIndexInvalid(name) => format!("`{}': not a valid index", name),
            ExecError::BadSubstitution(s) => format!("`{}': bad substitution", s),
            ExecError::DivZero => "divided by 0".to_string(),
            ExecError::Exponent(s) => format!("exponent less than 0 (error token is \"{}\")", s),
            ExecError::InvalidName(name) => format!("`{}': invalid name", name),
            ExecError::InvalidBase(b) => format!("sush: {0}: invalid arithmetic base (error token is \"{0}\")", b),
            ExecError::VariableReadOnly(name) => format!("{}: readonly variable", name),
            ExecError::VariableInvalid(name) => format!("`{}': not a valid identifier", name),
            ExecError::OperandExpected(name) => format!("`{}': syntax error: operand expected", name),
            ExecError::SubstringMinus(n) => format!("{}: substring expression < 0", n),
            ExecError::Other(name) => name,
        }
    }
}


pub fn print_e(e: ExecError, core: &mut ShellCore) {
    let name = core.db.get_param("0").unwrap();
    let s: String = From::<ExecError>::from(e);
    if core.db.flags.contains('i') {
        eprintln!("{}: {}", &name, &s);
    }else{
        let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
        eprintln!("{}: line {}: {}", &name, &lineno, s);
    }
}

#[derive(Debug)]
pub enum InputError {
    Interrupt,
    Eof,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedSymbol(String),
    UnexpectedEof,
    Interrupted,
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

/* error at wait */
pub fn signaled(pid: Pid, signal: Signal, coredump: bool) -> i32 {
    match coredump {
        true  => eprintln!("Pid: {:?}, Signal: {:?} (core dumped)", pid, signal),
        false => eprintln!("Pid: {:?}, Signal: {:?}", pid, signal),
    }
    128+signal as i32
}
