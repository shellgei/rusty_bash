//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::input::InputError;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedSymbol(String),
    Input(InputError),
    /*
    UnexpectedEof,
    Interrupted,
    */
}

impl From<ParseError> for String {
    fn from(e: ParseError) -> String {
        match e {
            ParseError::UnexpectedSymbol(s) => format!("Unexpected token: {}", s),
            ParseError::Input(e) => From::from(e),
            /*
            ParseError::UnexpectedEof => "syntax error: unexpected end of file".to_string(),
            ParseError::Interrupted => "interrupted".to_string(),
            */
        }
    }
}

pub fn print_error(e: ParseError, core: &mut ShellCore) {
    let name = core.db.get_param("0").unwrap();
    let s: String = From::<ParseError>::from(e);
    if core.db.flags.contains('i') {
        eprintln!("{}: {}", &name, &s);
    }else{
        let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
        eprintln!("{}: line {}: {}", &name, &lineno, s);
    }
}
