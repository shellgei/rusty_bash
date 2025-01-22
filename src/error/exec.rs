//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::parse::ParseError;

#[derive(Debug, Clone)]
pub enum ExecError {
    Internal,
    ArrayIndexInvalid(String),
    AssignmentToNonVariable(String),
    BadSubstitution(String),
    DivZero,
    Exponent(i64),
    InvalidBase(String),
    InvalidName(String),
    InvalidOption(String),
    Interrupted,
    ValidOnlyInFunction(String),
    VariableReadOnly(String),
    VariableInvalid(String),
    OperandExpected(String),
    ParseError(ParseError),
    Recursion(String),
    SubstringMinus(i64),
    Other(String),
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
            ExecError::ArrayIndexInvalid(name) => format!("`{}': not a valid index", name),
            ExecError::BadSubstitution(s) => format!("`{}': bad substitution", s),
            ExecError::DivZero => "divided by 0".to_string(),
            ExecError::Exponent(s) => format!("exponent less than 0 (error token is \"{}\")", s),
            ExecError::InvalidName(name) => format!("`{}': invalid name", name),
            ExecError::InvalidBase(b) => format!("{0}: invalid arithmetic base (error token is \"{0}\")", b),
            ExecError::InvalidOption(opt) => format!("{}: invalid option", opt),
            ExecError::Interrupted => "interrupted".to_string(),
            ExecError::AssignmentToNonVariable(right) => format!("attempted assignment to non-variable (error token is \"{}\")", right),
            ExecError::ValidOnlyInFunction(com) => format!("{}: can only be used in a function", &com),
            ExecError::VariableReadOnly(name) => format!("{}: readonly variable", name),
            ExecError::VariableInvalid(name) => format!("`{}': not a valid identifier", name),
            ExecError::OperandExpected(token) => format!("{0}: syntax error: operand expected (error token is \"{0}\")", token),
            ExecError::ParseError(p) => From::from(p),
            ExecError::Recursion(token) => format!("{0}: expression recursion level exceeded (error token is \"{0}\")", token), 
            ExecError::SubstringMinus(n) => format!("{}: substring expression < 0", n),
            ExecError::Other(name) => name.to_string(),
        }
    }
}

impl ExecError {
    pub fn print(&self, _: &mut ShellCore) {
        let s: String = From::from(self);
        eprintln!("{}", &s);
    }
}
