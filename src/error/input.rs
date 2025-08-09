//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug, Clone)]
pub enum InputError {
    NotUtf8,
    NoSuchFile(String),
    Interrupt,
    Eof,
}

impl From<&InputError> for String {
    fn from(e: &InputError) -> String {
        match e {
            InputError::NotUtf8 => "input error: illegal utf-8 character".to_string(),
            InputError::NoSuchFile(filename) => format!("{filename}: No such file or directory"),
            InputError::Eof => "syntax error: unexpected end of file".to_string(),
            InputError::Interrupt => "interrupted".to_string(),
        }
    }
}
