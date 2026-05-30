//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

#[derive(Debug, Clone)]
pub enum InputError {
    BinaryFile(String),
    NotUtf8,
    NoSuchFile(String),
    Interrupt,
    Timeout,
    Eof,
}

impl From<&InputError> for String {
    fn from(e: &InputError) -> String {
        match e {
            InputError::BinaryFile(filename) => format!("{filename}: cannot execute binary file"),
            InputError::NotUtf8 => "input error: illegal utf-8 character".to_string(),
            InputError::NoSuchFile(filename) => format!("{filename}: No such file or directory"),
            InputError::Eof => "syntax error: unexpected end of file".to_string(),
            InputError::Interrupt => "interrupted".to_string(),
            InputError::Timeout => "timeout".to_string(),
        }
    }
}

impl InputError {
    pub fn print(&self, core: &mut ShellCore) {
        let name = core.db.get_param("0").unwrap();
        let s: String = From::<&InputError>::from(self);
        if s == "" {
            return;
        }

        if core.db.flags.contains('i') {
            eprintln!("{}: {}", &name, &s);
        } else {
            let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
            eprintln!("{}: line {}: {}", &name, &lineno, s);
        }
    }
}
