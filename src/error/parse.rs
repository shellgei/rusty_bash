//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::input::InputError;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedSymbol(String),
    Input(InputError),
}

impl From<&ParseError> for String {
    fn from(e: &ParseError) -> String {
        match e {
            ParseError::UnexpectedSymbol(s) => format!("Unexpected token: {}", s),
            ParseError::Input(e) => From::from(e),
        }
    }
}

impl ParseError {
    pub fn print(&self, _: &mut ShellCore) {
        let s: String = From::from(self);
        eprintln!("{}", &s);
    }
}
