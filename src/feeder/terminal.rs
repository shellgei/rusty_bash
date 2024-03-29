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
    chars: Vec<Vec<char>>,
    insert_point_x: usize,
    insert_point_y: usize,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        print!("{} ", prompt);
        io::stdout().flush().unwrap();

        Terminal {
            prompt_len: UnicodeWidthStr::width(prompt),
            stdout: io::stdout().into_raw_mode().unwrap(),
            chars: vec![vec![]],
            insert_point_x: 0,
            insert_point_y: 0,
        }
    }

    pub fn insert(&mut self, c: &char) {
        self.chars[self.insert_point_y].insert(self.insert_point_x, *c);
        self.insert_point_x += 1;
    }

    pub fn get_string(&self) -> String {
        let mut ans = String::new();
        for line in &self.chars {
            ans.push_str(&line.iter().collect::<String>());
        }
        ans
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
            event::Key::Char('\n') => {
                write!(term.stdout, "\r\n").unwrap();
                break;
            },
            event::Key::Char(c) => {
                term.insert(c);
            },
            _  => {},
        }
    }
    Ok(term.get_string())
}
