//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod terminal;

use crate::core::ShellCore;
use std::io;

pub enum InputError {
    Eof,
    Interrupt,
}

pub struct Feeder {
    pub remaining: String,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder { remaining: String::new(), }
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
        let line = match core.has_flag('i') {
            true  => terminal::read_line(core, "PS1"),
            false => Self::read_line_stdin(),
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
