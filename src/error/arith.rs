//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug, Clone)]
pub enum ArithError {
    DivZero(String, String),
    Exponent(i128),
    InvalidBase(String),
    InvalidNumber(String),
    InvalidOperator(String, String),
    OperandExpected(String),
}

impl From<ArithError> for String {
    fn from(e: ArithError) -> String {
        Self::from(&e)
    }
}

impl From<&ArithError> for String {
    fn from(e: &ArithError) -> String {
        match e {
            ArithError::DivZero(expr, token) => format!("{}: division by 0 (error token is \"{}\")", expr, token),
            ArithError::Exponent(s) => format!("exponent less than 0 (error token is \"{}\")", s),
            ArithError::InvalidBase(b) => format!("{0}: invalid arithmetic base (error token is \"{0}\")", b),
            ArithError::InvalidNumber(name) => format!("{0}: invalid number (error token is \"{0}\")", name),
            ArithError::InvalidOperator(s, tok) => format!("{}: syntax error: invalid arithmetic operator (error token is \"{}\")", s, tok),
            ArithError::OperandExpected(token) => format!("{0}: syntax error: operand expected (error token is \"{0}\")", token),
        }
    }
}
