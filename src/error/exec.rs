//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

#[derive(Debug)]
pub enum ExecError {
    Internal,
    ArrayIndexInvalid(String),
    BadSubstitution(String),
    DivZero,
    Exponent(i64),
    InvalidBase(String),
    InvalidName(String),
    ValidOnlyInFunction(String),
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
            ExecError::ValidOnlyInFunction(com) => format!("{}: can only be used in a function", &com),
            ExecError::VariableReadOnly(name) => format!("{}: readonly variable", name),
            ExecError::VariableInvalid(name) => format!("`{}': not a valid identifier", name),
            ExecError::OperandExpected(token) => format!("{0}: syntax error: operand expected (error token is \"{0}\")", token),
            ExecError::SubstringMinus(n) => format!("{}: substring expression < 0", n),
            ExecError::Other(name) => name,
        }
    }
}

pub fn print_error(e: ExecError, core: &mut ShellCore) {
    let name = core.db.get_param("0").unwrap();
    let s: String = From::<ExecError>::from(e);
    if core.db.flags.contains('i') {
        eprintln!("{}: {}", &name, &s);
    }else{
        let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
        eprintln!("{}: line {}: {}", &name, &lineno, s);
    }
}
