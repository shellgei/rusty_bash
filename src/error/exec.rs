//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::error::arith::ArithError;
use crate::error::parse::ParseError;
use crate::ShellCore;
use nix::errno::Errno;
use nix::sys::wait::WaitStatus;
use std::num::ParseIntError;
use std::os::fd::RawFd;

#[derive(Debug, Clone)]
pub enum ExecError {
    Internal,
    ArgListTooLong(String),
    AmbiguousRedirect(String),
    ArrayIndexInvalid(String),
    BadSubstitution(String),
    BadFd(RawFd),
    Bug(String),
    CannotOverwriteExistingFile(String),
    CircularNameRef(String),
    CommandNotFound(String),
    InvalidIndirectExpansion(String),
    InvalidName(String),
    InvalidNameRef(String),
    InvalidOption(String),
    Interrupted,
    ValidOnlyInFunction,
    VariableReadOnly(String),
    VariableInvalid(String),
    ParseIntError(String),
    PermissionDenied(String),
    SelfRef(String),
    SyntaxError(String),
    Restricted(String),
    RefCannotBeArray(String),
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


//    command_error_exit(command_name, core, "Arg list too long", 126)
//    command_error_exit(command_name, core, "Permission denied", 126)

impl From<&ExecError> for String {
    fn from(e: &ExecError) -> String {
        match e {
            ExecError::Internal => "INTERNAL ERROR".to_string(),
            ExecError::AmbiguousRedirect(name) => format!("{name}: ambiguous redirect"),
            ExecError::ArrayIndexInvalid(name) => format!("[{name}]: bad array subscript"),
            ExecError::ArgListTooLong(name) => format!("{name}: Arg list too long"),
            ExecError::BadSubstitution(s) => format!("`{s}': bad substitution"),
            ExecError::BadFd(fd) => format!("{fd}: bad file descriptor"),
            ExecError::CannotOverwriteExistingFile(file) => {
                format!("{file}: cannot overwrite existing file")
            }
            ExecError::CircularNameRef(name) => format!("{name}: circular name reference"),
            ExecError::CommandNotFound(name) => format!("{name}: command not found"),
            ExecError::InvalidIndirectExpansion(name) => format!("{name}: invalid indirect expansion"),
            ExecError::InvalidName(name) => format!("`{name}': not a valid identifier"),
            ExecError::InvalidNameRef(name) => format!("`{name}': invalid variable name for name reference"),
            ExecError::InvalidOption(opt) => format!("{opt}: invalid option"),
            ExecError::Interrupted => "interrupted".to_string(),
            ExecError::ValidOnlyInFunction => "can only be used in a function".to_string(),
            ExecError::VariableReadOnly(name) => format!("{name}: readonly variable"),
            ExecError::VariableInvalid(name) => format!("`{name}': not a valid identifier"),
            ExecError::ParseIntError(e) => e.to_string(),
            ExecError::PermissionDenied(name) => format!("{name}: Permission denied"),
            ExecError::SelfRef(name) => {
                format!("{name}: nameref variable self references not allowed")
            },
            ExecError::SyntaxError(near) => {
                format!("syntax error near unexpected token `{}'", &near)
            }
            ExecError::Restricted(com) => format!("{com}: restricted"),
            ExecError::RefCannotBeArray(name) => format!("{name}: reference variable cannot be an array"),
            ExecError::SubstringMinus(n) => format!("{n}: substring expression < 0"),
            ExecError::UnsupportedWaitStatus(ws) => format!("Unsupported wait status: {ws:?}"),
            ExecError::UnboundVariable(name) => format!("{name}: unbound variable"),
            ExecError::Errno(e) => format!("system error {e:?}"),
            ExecError::Bug(msg) => format!("INTERNAL BUG: {msg}"),
            ExecError::Other(name) => name.to_string(),

            ExecError::ArithError(s, a) => format!("{}: {}", s, String::from(a)),
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
        } else {
            let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
            eprintln!("{}: line {}: {}", &name, &lineno, s);
        }
    }
}
