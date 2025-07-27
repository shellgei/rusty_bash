//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::input::InputError;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedSymbol(String),
    Input(InputError),
    WrongAlias(String),
    Regex(String),
}
//expected for conditional expression

impl From<&ParseError> for String {
    fn from(e: &ParseError) -> String {
        match e {
            //ParseError::UnexpectedSymbol(s) => format!("Unexpected token: {}", s),
            ParseError::UnexpectedSymbol(s) => format!("syntax error near unexpected token: {}", s),
            ParseError::Input(e) => From::from(e),
            ParseError::WrongAlias(msg) => format!("Something wrong alias: {}", msg),
            ParseError::Regex(msg) => format!("regex error: {}", msg),
        }
    }
}

impl ParseError {
    pub fn print(&self, core: &mut ShellCore) {
        let name = core.db.get_param("0").unwrap();
        let s: String = From::<&ParseError>::from(self);
        if core.db.flags.contains('i') {
            eprintln!("{}: {}", &name, &s);
        }else{
            let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
            eprintln!("{}: line {}: {}", &name, &lineno, s);
        }
    }
}
