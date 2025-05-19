//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug, Clone)]
pub enum ArithError {
    AssignmentToNonVariable(String),
    DivZero(String),
    Exponent(i128),
    InvalidBase(String),
    InvalidNumber(String),
    InvalidOperator(String, String),
    OperandExpected(String),
    SyntaxError(String),
}

impl From<ArithError> for String {
    fn from(e: ArithError) -> String {
        Self::from(&e)
    }
}

impl From<&ArithError> for String {
    fn from(e: &ArithError) -> String {
        match e {
            ArithError::AssignmentToNonVariable(right) => format!("attempted assignment to non-variable (error token is \"{}\")", right),
            ArithError::DivZero(token) => format!("division by 0 (error token is \"{}\")", token),
            ArithError::Exponent(s) => format!("exponent less than 0 (error token is \"{}\")", s),
            ArithError::InvalidBase(b) => format!("invalid arithmetic base (error token is \"{}\")", b),
            ArithError::InvalidNumber(name) => format!("invalid number (error token is \"{}\")", name),
            ArithError::InvalidOperator(s, tok) => format!("{}: syntax error: invalid arithmetic operator (error token is \"{}\")", s, tok),
            ArithError::OperandExpected(token) => format!("syntax error: operand expected (error token is \"{}\")", token),
            ArithError::SyntaxError(token) => format!("syntax error in expression (error token is \"{}\")", token),
        }
    }
}
