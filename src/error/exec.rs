//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::parse::ParseError;
use std::os::fd::RawFd;
use nix::errno::Errno;
use nix::sys::wait::WaitStatus;

#[derive(Debug, Clone)]
pub enum ExecError {
    Internal,
    AmbiguousRedirect(String),
    ArrayIndexInvalid(String),
    AssignmentToNonVariable(String),
    BadSubstitution(String),
    BadFd(RawFd),
    DivZero,
    Exponent(i64),
    InvalidBase(String),
    InvalidName(String),
    InvalidOption(String),
    Interrupted,
    ValidOnlyInFunction,
    VariableReadOnly(String),
    VariableInvalid(String),
    OperandExpected(String),
    ParseError(ParseError),
    Recursion(String),
    SubstringMinus(i64),
    UnsupportedWaitStatus(WaitStatus),
    Errno(Errno),
    Other(String),
}

impl From<Errno> for ExecError {
    fn from(e: Errno) -> ExecError {
        ExecError::Errno(e)
    }
}

impl From<ParseError> for ExecError {
    fn from(e: ParseError) -> ExecError {
        ExecError::ParseError(e)
    }
}

impl From<ExecError> for String {
    fn from(e: ExecError) -> String {
        Self::from(&e)
    }
}

impl From<&ExecError> for String {
    fn from(e: &ExecError) -> String {
        match e {
            ExecError::Internal => "INTERNAL ERROR".to_string(),
            ExecError::AmbiguousRedirect(name) => format!("{name}: ambiguous redirect"),
            ExecError::ArrayIndexInvalid(name) => format!("`{name}': not a valid index"),
            ExecError::BadSubstitution(s) => format!("`{s}': bad substitution"),
            ExecError::BadFd(fd) => format!("{fd}: bad file descriptor"),
            ExecError::DivZero => "divided by 0".to_string(),
            ExecError::Exponent(s) => format!("exponent less than 0 (error token is \"{s}\")"),
            ExecError::InvalidName(name) => format!("`{name}': invalid name"),
            ExecError::InvalidBase(b) => format!("{b}: invalid arithmetic base (error token is \"{b}\")"),
            ExecError::InvalidOption(opt) => format!("{opt}: invalid option"),
            ExecError::Interrupted => "interrupted".to_string(),
            ExecError::AssignmentToNonVariable(right) => format!("attempted assignment to non-variable (error token is \"{right}\")"),
            ExecError::ValidOnlyInFunction => "can only be used in a function".to_string(),
            ExecError::VariableReadOnly(name) => format!("{name}: readonly variable"),
            ExecError::VariableInvalid(name) => format!("`{name}': not a valid identifier"),
            ExecError::OperandExpected(token) => format!("{token}: syntax error: operand expected (error token is \"{token}\")"),
            ExecError::ParseError(p) => From::from(p),
            ExecError::Recursion(token) => format!("{token}: expression recursion level exceeded (error token is \"{token}\")"), 
            ExecError::SubstringMinus(n) => format!("{n}: substring expression < 0"),
            ExecError::UnsupportedWaitStatus(ws) => format!("Unsupported wait status: {ws:?}"),
            ExecError::Errno(e) => format!("system error {e:?}"),
            ExecError::Other(name) => name.to_string(),
        }
    }
}

impl ExecError {
    pub fn print(&self, _: &mut ShellCore) {
        let s: String = From::<&ExecError>::from(self);
        eprintln!("sush: {s}");
    }
}
