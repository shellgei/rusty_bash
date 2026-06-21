//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::input::InputError;
use crate::ShellCore;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedSymbol(String),
    Input(InputError),
    WrongAlias(String),
    SyntaxError(String),
}
//expected for conditional expression

impl From<&ParseError> for String {
    fn from(e: &ParseError) -> String {
        match e {
            //ParseError::UnexpectedSymbol(s) => format!("Unexpected token: {}", s),
            ParseError::UnexpectedSymbol(s) => format!("syntax error near unexpected token `{s}'"),
            ParseError::Input(e) => From::from(e),
            ParseError::WrongAlias(msg) => format!("Someting wrong alias: {msg}"),
            ParseError::SyntaxError(s) => format!("syntax error: {s}"),
        }
    }
}

impl ParseError {
    pub fn print(&self, core: &mut ShellCore) {
        let name = core.db.get_param("0").unwrap();
        let mut s: String = From::<&ParseError>::from(self);
        s = s.trim_end().to_string();

        let appear_c = matches!(self, Self::UnexpectedSymbol(_) | Self::SyntaxError(_));

        if core.db.flags.contains('i') {
            eprintln!("{}: {}", &name, &s);
        } else if core.db.flags.contains('c') && appear_c {
            let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
            eprintln!("{}: -c: line {}: {}", &name, &lineno, s);
            if !core.case_line.is_empty() {
                eprintln!(
                    "{}: -c: line {}: `{}'",
                    &name,
                    &lineno,
                    &core.case_line.trim_end()
                );
                core.case_line.clear();
            }
        } else {
            let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
            eprintln!("{}: line {}: {}", &name, &lineno, s);
            if !core.case_line.is_empty() {
                eprintln!(
                    "{}: line {}: `{}'",
                    &name,
                    &lineno,
                    &core.case_line.trim_end()
                );
                core.case_line.clear();
            }
        }
    }
}
