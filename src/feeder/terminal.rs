//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{InputError, ShellCore};
use std::io;
use std::io::{Write, Stdout};
use termion::event;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;
use unicode_width::UnicodeWidthStr;

struct Terminal {
    prompt_len: usize,
    stdout: RawTerminal<Stdout>,
    chars: Vec<char>,
    insert_point: usize,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        Terminal {
            prompt_len: UnicodeWidthStr::width(prompt),
            stdout: io::stdout().into_raw_mode().unwrap(),
            chars: vec![],
            insert_point: 0,
        }
    }

    pub fn insert(&mut self, c: &char) {
        self.chars.insert(self.insert_point, *c);
        self.insert_point += 1;
        write!(self.stdout, "{}", *c).unwrap();
        self.stdout.flush().unwrap();
        //eprintln!("{:?}", self.stdout.cursor_pos().unwrap());
    }

    pub fn get_string(&self) -> String {
        self.chars.iter().collect::<String>()
    }
}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError>{
    let mut term = Terminal::new(core, prompt);

    for c in io::stdin().keys() {
        match c.as_ref().unwrap() {
            event::Key::Ctrl('c') => {
                write!(term.stdout, "^C\r\n").unwrap();
                return Err(InputError::Interrupt);
            },
            event::Key::Char(c) => {
                term.insert(c);
                if *c == '\n' {
                    break;
                }
            },
            _  => {},
        }
    }
    write!(term.stdout, "\r").unwrap();
    Ok(term.get_string())
}
