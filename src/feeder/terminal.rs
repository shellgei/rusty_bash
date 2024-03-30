//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{InputError, ShellCore};
use std::io;
use std::io::{Write, Stdout};
use termion::event;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;

struct Terminal {
    prompt: String,
    stdout: RawTerminal<Stdout>,
    chars: Vec<char>,
    insert_pos: usize,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        Terminal {
            prompt: prompt.to_string(),
            stdout: io::stdout().into_raw_mode().unwrap(),
            chars: prompt.chars().collect(),
            insert_pos: prompt.chars().count(),
        }
    }

    pub fn insert(&mut self, c: char) {
        self.chars.insert(self.insert_pos, c);
        self.insert_pos += 1;
        write!(self.stdout, "{}", c).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn get_string(&self) -> String {
        let cut = self.prompt.chars().count();
        self.chars[cut..].iter().collect()
    }

    pub fn goto_origin(&mut self) {
        self.insert_pos = self.prompt.chars().count();
    }
}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError>{
    let mut term = Terminal::new(core, prompt);

    for c in io::stdin().keys() {
        match c.as_ref().unwrap() {
            event::Key::Ctrl('a') => {
                term.goto_origin();
            },
            event::Key::Ctrl('c') => {
                write!(term.stdout, "^C\r\n").unwrap();
                return Err(InputError::Interrupt);
            },
            event::Key::Ctrl('d') => {
                write!(term.stdout, "\r\n").unwrap();
                return Err(InputError::Eof);
            },
            event::Key::Char('\n') => {
                write!(term.stdout, "\r\n").unwrap();
                term.chars.push('\n');
                break;
            },
            event::Key::Char(c) => {
                term.insert(*c);
            },
            _  => {},
        }
    }
    Ok(term.get_string())
}
