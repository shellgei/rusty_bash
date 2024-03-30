//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{InputError, ShellCore};
use std::io;
use std::io::{Write, Stdout};
use termion::cursor::DetectCursorPos;
use termion::event;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;
use unicode_width::UnicodeWidthStr;

struct Terminal {
    prompt: String,
    stdout: RawTerminal<Stdout>,
    chars: Vec<char>,
    insert_pos: usize,
    prompt_row: u16,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut term = Terminal {
            prompt: prompt.to_string(),
            stdout: io::stdout().into_raw_mode().unwrap(),
            chars: prompt.chars().collect(),
            insert_pos: prompt.chars().count(),
            prompt_row: 0,
        };

        term.prompt_row = term.stdout.cursor_pos().unwrap().1;

        term
    }

    fn cursor_pos(&self, ins_pos: usize) -> (u16, u16) {
        let s = self.chars[..ins_pos].iter().collect::<String>();
        let x = UnicodeWidthStr::width(&s[0..]) + 1;

        (x.try_into().unwrap(), self.prompt_row)
    }

    pub fn insert(&mut self, c: char) {
        self.chars.insert(self.insert_pos, c);
        self.insert_pos += 1;

        if self.insert_pos == self.chars.len() {
            write!(self.stdout, "{}", c).unwrap();
        }else{
            let prompt_pos = self.cursor_pos(self.prompt.chars().count());
            let pos = self.cursor_pos(self.insert_pos);

            write!(self.stdout, "{}{}{}",
                   termion::cursor::Goto(prompt_pos.0, prompt_pos.1),
                   self.get_string(),
                   termion::cursor::Goto(pos.0, pos.1),
            ).unwrap();
        }

        self.stdout.flush().unwrap();
    }

    pub fn get_string(&self) -> String {
        let cut = self.prompt.chars().count();
        self.chars[cut..].iter().collect()
    }

    pub fn goto_origin(&mut self) {
        self.insert_pos = self.prompt.chars().count();
        let pos = self.cursor_pos(self.insert_pos);
        write!(self.stdout, "{}",
               termion::cursor::Goto(pos.0, pos.1),
        ).unwrap();
        self.stdout.flush().unwrap();
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
