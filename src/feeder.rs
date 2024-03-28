//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::ShellCore;
use crate::terminal::Terminal;
use std::io;

pub enum InputError {
    Eof,
}

pub struct Feeder {
    remaining: String,
    term: Option<Terminal>,
}

impl Feeder {
    pub fn new(core: &ShellCore) -> Feeder {
        Feeder {
            remaining: String::new(),
            term: if core.has_flag('i') { Some(Terminal::new()) } else { None },
        }
    }

    fn read_line_stdin() -> Result<String, InputError> {
        let mut line = String::new();

        match io::stdin().read_line(&mut line) {
            Ok(0) => Err(InputError::Eof),
            Ok(_) => Ok(line),
            Err(e) => panic!("sush: {}", &e),
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        let line = match self.term.as_mut() {
            Some(t) => {let _ = t.read_line_normal(core); Self::read_line_stdin()},
            _ => Self::read_line_stdin(),
        };

        match line {
            Ok(ln) => {
                self.remaining = ln;
                print!("{}", &self.remaining);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
}
