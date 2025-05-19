//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug, Clone)]
pub enum ArithError {
    AssignmentToNonVariable(String),
    DivZero(String),
    Exponent(i128),
    InvalidBase(String),
    InvalidNumber(String),
    InvalidOperator(String),
    OperandExpected(String),
    Recursion(String),
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
            ArithError::AssignmentToNonVariable(right)
                => error_msg("attempted assignment to non-variable", right),
            ArithError::DivZero(token)
                => error_msg("division by 0", token),
            ArithError::Exponent(s)
                => error_msg("exponent less than 0", &s.to_string()),
            ArithError::InvalidBase(b)
                => error_msg("invalid arithmetic base", b),
            ArithError::InvalidNumber(name)
                => error_msg("invalid number", name),
            ArithError::InvalidOperator(tok)
                => error_msg("invalid arithmetic operator", tok),
            ArithError::OperandExpected(token)
                => error_msg("syntax error: operand expected", token),
            ArithError::Recursion(token)
                => error_msg("expression recursion level exceeded", token), 
            ArithError::SyntaxError(token)
                => error_msg("syntax error in expression", token),
        }
    }
}

fn error_msg(msg: &str, token: &str) -> String {
    msg.to_string() + &format!(" (error token is \"{}\")", token)
}
