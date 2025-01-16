//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug)]
pub enum ParseError {
    UnexpectedSymbol(String),
    UnexpectedEof,
    Interrupted,
}

impl From<ParseError> for String {
    fn from(e: ParseError) -> String {
        match e {
            ParseError::UnexpectedSymbol(s) => format!("Unexpected token: {}", s),
            ParseError::UnexpectedEof => "syntax error: unexpected end of file".to_string(),
            ParseError::Interrupted => "interrupted".to_string(),
        }
    }
}
