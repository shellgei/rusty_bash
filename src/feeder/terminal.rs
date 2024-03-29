//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{InputError, ShellCore};
use std::io;
use std::io::{Write, stdout, stdin, Stdout};
use unicode_width::UnicodeWidthStr;

use termion::event;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;

pub struct Terminal {
    pub prompt_len: usize,
    pub stdout: RawTerminal<Stdout>,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, mode: &str) -> Terminal {
        let prompt = core.get_param_ref(mode);
        print!("{} ", prompt);
        io::stdout().flush().unwrap();
        let prompt_len = UnicodeWidthStr::width(prompt);

        Terminal {
            prompt_len: prompt_len,
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }

}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError>{
    let mut term = Terminal::new(core, prompt);

    for c in stdin().keys() {
        match &c.as_ref().unwrap() {
            event::Key::Ctrl('c') => {
                write!(term.stdout, "^C\r\n").unwrap();
                return Err(InputError::Interrupt);
            },
            event::Key::Char('\n') => {
                write!(term.stdout, "\r\n").unwrap();
                break;
            },
            _  => {},
        }
    }
    Ok("\n".to_string())
}
