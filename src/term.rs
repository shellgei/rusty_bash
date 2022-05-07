//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::{Write, stdout, stdin};
use termion::{event};
use termion::raw::IntoRawMode;
use termion::input::TermRead;

pub fn prompt(text: &String) {
    print!("{} $ ", text);
    io::stdout()
        .flush()
        .unwrap();
}

pub fn read_line() -> String{
    let mut line = "".to_string();

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        match c {
            Ok(event::Key::Ctrl('c')) => {
                line = "".to_string();
                write!(stdout, "^C\n").unwrap();
                break;
            },
            Ok(event::Key::Char(c)) => {
                    write!(stdout, "{}", c).unwrap();
                    line += &c.to_string();
                    stdout.flush().unwrap();
                    if c == '\n' {
                        break;
                    };
            },
            _ => {},
        }
    }
    write!(stdout, "\r").unwrap();
    stdout.flush().unwrap();
    line
}

