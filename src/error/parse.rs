//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::exec::ExecError;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedSymbol(String),
    UnexpectedEof,
    Interrupted,
}

impl From<ParseError> for ExecError {
    fn from(e: ParseError) -> ExecError {
        match e {
            ParseError::UnexpectedSymbol(s) => ExecError::Other(s),
            ParseError::UnexpectedEof => ExecError::Other("eof".to_string()),
            ParseError::Interrupted => ExecError::Other("Interrupted".to_string()),
        }
    }
}
