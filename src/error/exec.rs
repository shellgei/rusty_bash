//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::parse::ParseError;
use nix::errno::Errno;
use nix::sys::wait::WaitStatus;
use std::num::ParseIntError;
use std::os::fd::RawFd;

#[derive(Debug, Clone)]
pub enum ExecError {
    Internal,
    AmbiguousRedirect(String),
    ArrayIndexInvalid(String),
    AssignmentToNonVariable(String),
    BadSubstitution(String),
    BadFd(RawFd),
    Bug(String),
    DivZero(String, String),
    Exponent(i128),
    InvalidBase(String),
    InvalidArithmeticOperator(String, String),
    InvalidName(String),
    InvalidNumber(String),
    InvalidOption(String),
    Interrupted,
    ValidOnlyInFunction(String),
    VariableReadOnly(String),
    VariableInvalid(String),
    OperandExpected(String),
    ParseError(ParseError),
    ParseIntError(String),
    SyntaxError(String),
    Restricted(String),
    Recursion(String),
    SubstringMinus(i128),
    UnsupportedWaitStatus(WaitStatus),
    Errno(Errno),
    Other(String),
}

impl From<Errno> for ExecError {
    fn from(e: Errno) -> ExecError {
        ExecError::Errno(e)
    }
}

impl From<ParseIntError> for ExecError {
    fn from(e: ParseIntError) -> ExecError {
        ExecError::ParseIntError(e.to_string())
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
            ExecError::AmbiguousRedirect(name) => format!("{}: ambiguous redirect", name),
            ExecError::ArrayIndexInvalid(name) => format!("`{}': not a valid index", name),
            ExecError::BadSubstitution(s) => format!("`{}': bad substitution", s),
            ExecError::BadFd(fd) => format!("{}: bad file descriptor", fd),
            ExecError::DivZero(expr, token) => format!("{}: division by 0 (error token is \"{}\"", expr, token),
            ExecError::Exponent(s) => format!("exponent less than 0 (error token is \"{}\")", s),
            ExecError::InvalidName(name) => format!("`{}': invalid name", name),
            ExecError::InvalidNumber(name) => format!("`{}': invalid number", name),
            //ExecError::InvalidIdentifier(name) => format!("`{}': not a valid identifier", name),
            ExecError::InvalidBase(b) => format!("{0}: invalid arithmetic base (error token is \"{0}\")", b),
            ExecError::InvalidArithmeticOperator(s, tok) => format!("{}: syntax error: invalid arithmetic operator (error token is \"{}\")", s, tok),
            ExecError::InvalidOption(opt) => format!("{}: invalid option", opt),
            ExecError::Interrupted => "interrupted".to_string(),
            ExecError::AssignmentToNonVariable(right) => format!("attempted assignment to non-variable (error token is \"{}\")", right),
            ExecError::ValidOnlyInFunction(com) => format!("{}: can only be used in a function", &com),
            ExecError::VariableReadOnly(name) => format!("{}: readonly variable", name),
            ExecError::VariableInvalid(name) => format!("`{}': not a valid identifier", name),
            ExecError::OperandExpected(token) => format!("{0}: syntax error: operand expected (error token is \"{0}\")", token),
            ExecError::ParseError(p) => From::from(p),
            ExecError::ParseIntError(e) => e.to_string(),
            ExecError::SyntaxError(near) => format!("syntax error near {}", &near),
            ExecError::Recursion(token) => format!("{0}: expression recursion level exceeded (error token is \"{0}\")", token), 
            ExecError::Restricted(com) => format!("{}: restricted", com), 
            ExecError::SubstringMinus(n) => format!("{}: substring expression < 0", n),
            ExecError::UnsupportedWaitStatus(ws) => format!("Unsupported wait status: {:?}", ws),
            ExecError::Errno(e) => format!("system error {:?}", e),
            ExecError::Bug(msg) => format!("INTERNAL BUG: {}", msg),
            ExecError::Other(name) => name.to_string(),
        }
    }
}

impl ExecError {
    pub fn print(&self, core: &mut ShellCore) {
        let name = core.db.get_param("0").unwrap();
        let s: String = From::<&ExecError>::from(self);
        if core.db.flags.contains('i') {
            eprintln!("{}: {}", &name, &s);
        }else{
            let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
            eprintln!("{}: line {}: {}", &name, &lineno, s);
        }
    }
}
