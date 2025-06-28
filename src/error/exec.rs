//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::arith::ArithError;
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
    BadSubstitution(String),
    BadFd(RawFd),
    Bug(String),
    CannotOverwriteExistingFile(String),
    InvalidName(String),
    InvalidOption(String),
    Interrupted,
    ValidOnlyInFunction(String),
    VariableReadOnly(String),
    VariableInvalid(String),
    ParseIntError(String),
    SyntaxError(String),
    Restricted(String),
    SubstringMinus(i128),
    UnsupportedWaitStatus(WaitStatus),
    UnboundVariable(String),
    Errno(Errno),
    Other(String),

    ParseError(ParseError),
    ArithError(String, ArithError),
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

impl From<ArithError> for ExecError {
    fn from(e: ArithError) -> ExecError {
        ExecError::ArithError(String::new(), e)
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
            ExecError::CannotOverwriteExistingFile(file) => format!("{}: cannot overwrite existing file", file),
            //ExecError::InvalidName(name) => format!("`{}': invalid name", name),
            ExecError::InvalidName(name) => format!("`{}': not a valid identifier", name),
            ExecError::InvalidOption(opt) => format!("{}: invalid option", opt),
            ExecError::Interrupted => "interrupted".to_string(),
            ExecError::ValidOnlyInFunction(com) => format!("{}: can only be used in a function", &com),
            ExecError::VariableReadOnly(name) => format!("{}: readonly variable", name),
            ExecError::VariableInvalid(name) => format!("`{}': not a valid identifier", name),
            ExecError::ParseIntError(e) => e.to_string(),
            ExecError::SyntaxError(near) => format!("syntax error near unexpected token `{}'", &near),
            ExecError::Restricted(com) => format!("{}: restricted", com), 
            ExecError::SubstringMinus(n) => format!("{}: substring expression < 0", n),
            ExecError::UnsupportedWaitStatus(ws) => format!("Unsupported wait status: {:?}", ws),
            ExecError::UnboundVariable(name) => format!("{}: unbound variable", name),
            ExecError::Errno(e) => format!("system error {:?}", e),
            ExecError::Bug(msg) => format!("INTERNAL BUG: {}", msg),
            ExecError::Other(name) => name.to_string(),

            ExecError::ArithError(s, a) =>  format!("{}: {}", s, String::from(a)),
            ExecError::ParseError(p) => From::from(p),
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
